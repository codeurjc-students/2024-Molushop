use actix_web::Scope;
use actix_web::{get, post, web,delete, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use super::*;

pub static SCOPE_ES: &str = "/es";

use crate::middleware::auth::Auth;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;
/* 
pub fn scope_es() -> Scope{

}
*/

//Aquí habrá un scpope sobre el idioma "es"
use actix_web::{
    dev::{ServiceFactory,forward_ready, Service, ServiceRequest, ServiceResponse},
    Error,
};

pub fn scope_es(pool:web::Data<DbPool>) -> Scope<impl ServiceFactory<ServiceRequest, Config = (), Response = ServiceResponse, Error = actix_web::Error, InitError = ()>>{
    web::scope(SCOPE_ES)
        .wrap(Auth::new())
        .configure(home_page::config)
        .configure(about_us::config)
        .configure(cart_page::config)
        .configure(product_page::config)
}