use actix_web::{get, post, web,delete,Scope, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use lazy_static::lazy_static;
use askama::Template;
use uuid::Uuid;

use crate::schema::seller;
use crate::services::servicesX::{get_seller_products};

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
//type DbPool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
type DbPool = Pool<AsyncPgConnection>;

static SCOPE: &str = "/test";


pub fn scope() -> Scope{
    web::scope(SCOPE)
    .configure(config)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(seller_products);
}


#[get("/get-seller-products")]
async fn seller_products(pool: web::Data<DbPool>) -> impl Responder {
    let user_id = Uuid::parse_str("2064d62a-4978-4fe7-bef2-7690ff09bdc8").unwrap();
    let products = get_seller_products(&user_id, &pool).await.unwrap();
    HttpResponse::Ok().json(products)
}

//hacer endpoint para insertar muchos productos de prueba
