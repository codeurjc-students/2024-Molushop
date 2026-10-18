use actix_web::{get, post, web,delete,Scope, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use diesel::result;
use lazy_static::lazy_static;
use askama::Template;
use uuid::Uuid;

use crate::{models::data_transfer_objects::product::RoutesProductPanelGroup as Routes, schema::admins::id};
use crate::models::components::confirm_modal::ConfirmModal;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;

use super::scope::SCOPE_PANEL_COMPONENTS;
use super::panel_access::product_access;
use crate::services::panel_role_service::PanelUser;
use crate::services::products_panel::delete_product as delete_product_service;
type DbPool = Pool<AsyncPgConnection>;

static SCOPE: &str = "/product-panel-group";
static DELETE_PRODUCT : &str = "/delete-product";
lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza
    
    pub static ref ROUTES: Routes = Routes{
        delete_product: Box::leak(format!("{}{}/delete-product",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
        delete_product_modal: Box::leak(format!("{}{}/delete-product-modal",SCOPE_PANEL_COMPONENTS, SCOPE).into_boxed_str()),
    };

}

pub fn scope() -> Scope {
    web::scope(SCOPE)
        .configure(config)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    //cfg.service(products_panel_prueba);
    cfg
        .service(delete_product)
        .service(delete_product_modal);
        //.route(format!("{}/{}",DELETE_PRODUCT, "{id}"), web::get().to(products_panel));

}


#[delete("/delete-product/{id}")]
async fn delete_product(path: web::Path<Uuid>,panel_user: web::ReqData<PanelUser>,pool_data: web::Data<DbPool>) -> HttpResponse {
    if let Err(resp) = product_access(&panel_user, &path, pool_data.get_ref()).await {
        return resp;
    }
     
    let product_id = path.into_inner();
    // El vendedor solo borra lo de sus tiendas; el admin, cualquiera.
    let owner = if panel_user.is_admin {
        None
    } else {
        let store_ids = panel_user.manageable_store_ids();
        if store_ids.is_empty() {
            return HttpResponse::NotFound().finish();
        }
        Some(store_ids)
    };
    // No borra la fila: lo marca como borrado y lo quita de los carritos.
    let result = delete_product_service(&product_id,owner.as_deref(), pool_data.get_ref()).await;
    match result{
        Ok(true) => {
            HttpResponse::Ok().finish()
        },
        Ok(false) => HttpResponse::NotFound().finish(),
        Err(e) => {
            println!("Error! -> {}",e);
            return HttpResponse::InternalServerError().json("Error al eliminar el producto");}
        }
}

#[get("/delete-product-modal/{id}")]
async fn delete_product_modal(path: web::Path<Uuid>,panel_user: web::ReqData<PanelUser>,pool_data: web::Data<DbPool>) -> HttpResponse {
    let product_id = path.into_inner();
    if let Err(resp) = product_access(&panel_user, &product_id, pool_data.get_ref()).await {
        return resp;
    }

    let product_id_str= product_id.to_string();
    let warning_modal = ConfirmModal{
        method: "hx-delete".to_string(),
        message: "¿Estás seguro de que deseas eliminar este producto?".to_string(),
        endpoint: format!("{}/{}",ROUTES.delete_product,&product_id_str),
        target: format!("#card-{}",product_id_str),
        swap: "outerHTML swap:1s".to_string(),
        hyperscript_action:String::new(),
        htmx_active:true
    };
    let render = warning_modal.render().unwrap();
    HttpResponse::Ok().body(render)
}