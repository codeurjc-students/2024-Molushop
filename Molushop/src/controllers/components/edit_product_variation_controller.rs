use actix_web::{get, post, web,delete,Scope, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use diesel::result;
use lazy_static::lazy_static;
use rinja::Template;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

//use crate::{models::data_transfer_objects::product::RoutesProductPanelGroup as Routes, schema::admins::id};
use crate::{controllers::createProduct::product, models::components::warning_modal::WarningModal};
use crate::models::components::edit_product_variation_model::Routes;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use super::scope::SCOPE_COMPONENTS;
use crate::services::servicesX::{set_variation_identifiers};

use std::sync::Arc;
use crate::services::aws::s3::client::Client;
use actix_multipart::{
    form::{
        tempfile::{TempFile, TempFileConfig},
        MultipartForm,
        json::Json as MpJson
    },
    Multipart,
};

static SCOPE: &str = "/edit-product-variation";

lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza
    
    pub static ref ROUTES: Routes = Routes{
        //delete_product: Box::leak(format!("{}{}/delete-product",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
        //delete_product_modal: Box::leak(format!("{}{}/delete-product-modal",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
        edit_general: Box::leak(format!("{}{}/edit-general",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
        new_identifier: Box::leak(format!("{}{}/new-identifier",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
        edit_identifiers: Box::leak(format!("{}{}/edit-identifiers",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
        warning: Box::leak(format!("{}{}/pop-warning",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
        delete_identifier: Box::leak(format!("{}{}/delete-identifier",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
        //edit_images: Box::leak(format!("{}{}/edit-images",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
    };

}

pub fn scope() -> Scope {
    web::scope(SCOPE)
        .configure(config)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    //cfg.service(products_panel_prueba);
    cfg
        .service(spawn)
        .service(edit_identifiers)
        .service(pop_warning)
        .service(new_identifier)
        .service(delete_identifier);
        //.service(delete_product_modal);
        //.route(format!("{}/{}",DELETE_PRODUCT, "{id}"), web::get().to(products_panel));

}
use crate::models::components::edit_product_variation_model::EditProductVariation;
use crate::services::components::edit_product_variation_service::start;
use crate::models::components::modal_1::Modal1;

#[get("/spawn/{id}")]
async fn spawn(path: web::Path<Uuid>, pool_data: web::Data<DbPool>) -> HttpResponse {
    let variation_id = path.into_inner();
    let pool = pool_data.get_ref();
    //hacer un uuid de prueba de user
    //hacer una función que le pase todos los datos de la variación y me devuelva el template
    let seller_prueba = Uuid::parse_str("2064d62a-4978-4fe7-bef2-7690ff09bdc8").unwrap();
    let result_start=  start(&seller_prueba,&variation_id,&pool).await;
    match result_start{
        Ok(product_variation)=>{
            //tengo el ProductVariationEdit
            //ahora a cargar el producto con las rutas destinadas
            let edit_product_variation = EditProductVariation{
                variation:product_variation,
                routes_edit_product:&ROUTES
            }.render().unwrap();
            return HttpResponse::Ok().body(edit_product_variation)
        },
        Err(e)=>{
            println!("Error: {}",e);
            let modal_render = Modal1{
                text:"Ha ocurrido un error!".to_string()
            }.render().unwrap();
            return HttpResponse::InternalServerError().body(modal_render)
        }
    }
}

//parametro con el endpoint correspondiente??
#[get("/new-identifier")]
async fn new_identifier(pool_data:web::Data<DbPool>)->HttpResponse{
    use crate::models::components::identifier_model::IdentifierBase;
    let pool = pool_data.get_ref();
    let identifier_x = IdentifierBase::base(pool).await;
    let render = identifier_x.render().unwrap();
    return HttpResponse::Ok().body(render);
}

/* 
pub struct FormIdentifier{
    identifiers: Vec<Identifier>
} 

pub struct Identifier{
    name: String,
    value: String
}*/

use crate::models::components::edit_product_variation_model::Identifier;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdentifersForm{
    pub identifiers: Vec<Identifier>,
}

#[post("/edit-identifiers/{id_variation}")]
async fn edit_identifiers(pool_data: web::Data<DbPool>,path:web::Path<Uuid>,data: web::Json<IdentifersForm>)->HttpResponse{
    //nos van a pasar en el form un json
    let var_id = path.into_inner();
    let pool =pool_data.get_ref();
    let form_identifiers = data.into_inner();
    let identifiers = form_identifiers.identifiers;
    let result =set_variation_identifiers(&identifiers, &var_id, pool).await;
    match result{
        Ok(numero) =>{
            println!("Todo correcto");
        },
        Err(e)=>{
            println!("Ha ocurrido un error: {}",e);
        }
    }

    //ahora falta editar en la base de datos
    HttpResponse::Ok().finish()
}
//obtener los parametros 
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WarningParameters{
    pub method:String,
    pub message:String,
    pub endpoint:String,
    pub target:String,
    pub swap:String,
    pub hyperscript_action:String,
    pub htmx_active:bool
}


#[get("/pop-warning")]
async fn pop_warning(params: web::Query<WarningParameters>)->HttpResponse{
    //obtener el warning
    /* 
    let warning_modal = WarningModal{
        method:params.method.clone(),
        message:"Seguro que quieres eliminar el identificador?".to_string(),
        endpoint:params.endpoint.clone(),
        target:params.target.clone(),
        swap:params.swap.clone(),
        htmx_active:false,
    }.render().unwrap();
    remove target
    */
    println!("{:?}",params);
    let warning_modal = WarningModal{
        method:params.method.clone(),
        message:params.message.clone(),
        endpoint:params.endpoint.clone(),
        target:params.target.clone(),
        swap:params.swap.clone(),
        hyperscript_action: params.hyperscript_action.clone(), 
        htmx_active:params.htmx_active,
    }.render().unwrap();

    HttpResponse::Ok().body(warning_modal)
}

#[delete("/delete-identifier/{id_variation}")]
async fn delete_identifier(pool_data:web::Data<DbPool>)->HttpResponse{
    println!("BORRANDO IDENTIFICADOR");
    HttpResponse::Ok().finish()
}

/* 
#[post("/create-product/{category_id}")]
async fn create_product(pool_data: web::Data<DbPool>,path:web::Path<String>,data: web::Json<ProductForm>) -> impl Responder {
*/