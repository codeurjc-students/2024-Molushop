use actix_web::Scope;
use actix_web::{get, post, web,delete, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use super::es::*;

pub static SCOPE_MASTER: &str = "/master";



pub fn scope_master() -> Scope{
    web::scope(SCOPE_MASTER)
        .service(scope_es::scope_es())
}