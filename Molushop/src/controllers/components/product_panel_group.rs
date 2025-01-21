use actix_web::{get, post, web,delete,Scope, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use diesel::result;
use lazy_static::lazy_static;
use rinja::Template;
use uuid::Uuid;

use crate::models::data_transfer_objects::product::RoutesProductPanelGroup as Routes;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;

use super::scope::SCOPE_COMPONENTS;
use crate::services::servicesX::{delete_seller_product};
type DbPool = Pool<AsyncPgConnection>;

static SCOPE: &str = "/product-panel-group";
static DELETE_PRODUCT : &str = "/delete-product";
lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza
    
    pub static ref ROUTES: Routes = Routes{
        delete_product: Box::leak(format!("{}{}/delete-product",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
    };

}

pub fn scope() -> Scope {
    web::scope(SCOPE)
        .configure(config)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    //cfg.service(products_panel_prueba);
    cfg
        .service(delete_product);
        //.route(format!("{}/{}",DELETE_PRODUCT, "{id}"), web::get().to(products_panel));

}


#[delete("/delete-product/{id}")]
async fn delete_product(path: web::Path<Uuid>,pool_data: web::Data<DbPool>) -> HttpResponse {
    let product_id = path.into_inner();
    let seller_prueba = Uuid::parse_str("2064d62a-4978-4fe7-bef2-7690ff09bdc8").unwrap();
    // Lógica para borrar el producto
    let result = delete_seller_product(&product_id,&seller_prueba, pool_data.get_ref()).await;
    match result{
        Ok(_) => {
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            println!("Error! -> {}",e);
            return HttpResponse::InternalServerError().json("Error al eliminar el producto");}
        }
}