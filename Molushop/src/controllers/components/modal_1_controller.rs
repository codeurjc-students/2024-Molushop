use actix_web::{get, post, web,delete,Scope, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use diesel::result;
use lazy_static::lazy_static;
use rinja::Template;
use uuid::Uuid;

//use crate::{models::data_transfer_objects::product::RoutesProductPanelGroup as Routes, schema::admins::id};
use crate::models::components::warning_modal::WarningModal;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;

use crate::models::components::modal_1_model::Routes;

use super::scope::SCOPE_COMPONENTS;
use crate::services::servicesX::{delete_seller_product};
type DbPool = Pool<AsyncPgConnection>;

static SCOPE: &str = "/product-panel-group";
static DELETE_PRODUCT : &str = "/delete-product";
lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza
    
    pub static ref ROUTES: Routes = Routes{
        base: Box::leak(format!("{}{}/spawn",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
    };    

}

pub fn scope() -> Scope{
    web::scope(SCOPE)
        .configure(config)
}

pub fn config(cfg: &mut web::ServiceConfig){
    cfg 
        .service(spawn);
}

#[get("/spawn")]
async fn spawn() -> HttpResponse{
    HttpResponse::Ok().finish()
}