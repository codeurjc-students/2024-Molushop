use actix_web::Scope;
use actix_web::{get, post, web,delete, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use super::*;

pub static SCOPE_ES: &str = "/es";

/* 
pub fn scope_es() -> Scope{

}
*/

//Aquí habrá un scpope sobre el idioma "es"

pub fn scope_es() -> Scope{
    web::scope(SCOPE_ES)
        .configure(home_page::config)
        .configure(about_us::config)
}