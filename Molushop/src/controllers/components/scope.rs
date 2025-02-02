use super::{product_panel_group, edit_product};
use actix_web::Scope;
use actix_web::{get, post, web,delete, App, HttpResponse, HttpServer, Responder, http::StatusCode};


pub static SCOPE_COMPONENTS: &str = "/components";

pub fn scope_components() -> Scope {
    web::scope(SCOPE_COMPONENTS)
        .service(product_panel_group::scope())
        .service(edit_product::scope())
}