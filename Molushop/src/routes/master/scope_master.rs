use actix_web::Scope;
use actix_web::{get, post, web,delete, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use super::es::*;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

pub static SCOPE_MASTER: &str = "/master";



pub fn scope_master(pool:web::Data<DbPool>) -> Scope{
    web::scope(SCOPE_MASTER)
        .service(scope_es::scope_es(pool))
}