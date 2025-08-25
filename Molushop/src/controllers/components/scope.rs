use super::{product_panel_group, edit_product_controller,edit_product_variation_controller,login_base_controller};
use actix_web::Scope;
use actix_web::{get, post, web,delete, App, HttpResponse, HttpServer, Responder, http::StatusCode};


pub static SCOPE_COMPONENTS: &str = "/components";

pub fn scope_components() -> Scope {
    web::scope(SCOPE_COMPONENTS)
        .service(product_panel_group::scope())
        .service(edit_product_controller::scope())
        .service(edit_product_controller::scope())
        .service(edit_product_variation_controller::scope())
        .service(login_base_controller::scope1())
        .service(login_base_controller::scope2())
}