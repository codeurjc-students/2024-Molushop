use actix_web::{cookie::{time::OffsetDateTime, Cookie}, delete, dev::Transform, get, http::StatusCode, post, web, App, HttpResponse, HttpServer, Responder, Scope};
use lazy_static::lazy_static;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::models::components::nav1_model::*;
use super::scope::SCOPE_COMPONENTS;
use askama::Template;
use crate::services::components::nav1_service::*;

static SCOPE: &str = "/nav1";

lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza
    
    pub static ref ROUTES: Routes = Routes{
        //delete_product: Box::leak(format!("{}{}/delete-product",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
        //delete_product_modal: Box::leak(format!("{}{}/delete-product-modal",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
        home: "/home".to_string(),
        cart: "/cart".to_string()
        //login: Box::leak(format!("{}{}/login",SCOPE_COMPONENTS, SCOPE1).into_boxed_str())
    };

}

pub fn scope() -> Scope {
    web::scope(SCOPE)
    //.wrap(SayHi::new()) SI quiero meter middeware
    .configure(config)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    //cfg.service(products_panel_prueba);
    cfg
        .service(spawn);
}

#[get("/spawn")]
async fn spawn(pool_data: web::Data<DbPool>) -> HttpResponse {
    let render_login = get_nav1_render("Ejemplo".to_string(), None, pool_data.get_ref()).await;
    HttpResponse::Ok().body(render_login)
}
