use actix_web::{get, post, web,delete, App, HttpResponse,HttpRequest, HttpServer, Responder, http::StatusCode};
use lazy_static::lazy_static;
use uuid::Uuid;
use crate::services::components::nav1_service::*;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;
use rinja::Template;

pub fn config(cfg: &mut web::ServiceConfig) {
    //cfg.service(products_panel_prueba);
    cfg
        .service(get_about_us)
        ;
        //.route(format!("{}/{}",DELETE_PRODUCT, "{id}"), web::get().to(products_panel));

}

use crate::models::pages_models::master::es::about_us::*;

#[get("/about-us")]
async fn get_about_us(pool_data:web::Data<DbPool>)-> HttpResponse{
    let mut user_logged = false;
    
    //que componentes colocar=
    let about_us_render = AboutUs{
        user_logged,
        page_name:"About us".to_string(),
        nav1:get_nav1_object("hola".to_string())
    }.render().unwrap();

    HttpResponse::Ok().body(about_us_render)
}