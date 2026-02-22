use actix_web::{cookie::{time::OffsetDateTime, Cookie}, delete, dev::Transform, get, http::StatusCode, post, web, App, HttpResponse, HttpServer, Responder, Scope};
use lazy_static::lazy_static;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::models::components::nav1_model::*;
use super::scope::SCOPE_COMPONENTS;
use askama::Template;

static SCOPE: &str = "/nav1";

lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza
    
    // pub static ref ROUTES: Routes = Routes{
    //     //delete_product: Box::leak(format!("{}{}/delete-product",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
    //     //delete_product_modal: Box::leak(format!("{}{}/delete-product-modal",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
    //     home: Box::leak(format!("{}{}/home???",SCOPE_COMPONENTS, SCOPE2).into_boxed_str()),
    // };

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
async fn spawn()->HttpResponse	{
    //service que me cargue datos del login
    //parámetros que desean --> 
    let render_login = Nav1{}.render().unwrap();
    HttpResponse::Ok().body(render_login)
}
