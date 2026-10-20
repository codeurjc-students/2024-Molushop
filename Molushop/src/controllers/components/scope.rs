use super::{product_card_group_controller, product_panel_group, edit_product_controller,edit_product_variation_controller,login_base_controller,cart_add_controller,checkout_controller,product_reviews_controller};
use actix_web::Scope;
use actix_web::{get, post, web,delete, App, HttpResponse, HttpServer, Responder, http::StatusCode};


pub static SCOPE_COMPONENTS: &str = "/components";
/// Componentes del panel de productos. Se registran dentro del scope `/panel`
/// (`routes/panel.rs`), detrás de `Auth` + `PanelGuard`.
pub static SCOPE_PANEL_COMPONENTS: &str = "/panel/components";

pub fn scope_components() -> Scope {
    web::scope(SCOPE_COMPONENTS)
        .service(login_base_controller::scope1())
        .service(login_base_controller::scope2())
        .service(cart_add_controller::scope())
        .service(checkout_controller::scope())
        .service(product_reviews_controller::scope())
        .service(product_card_group_controller::scope())
}

/// Relativo a `/panel`: sirve `SCOPE_PANEL_COMPONENTS`.
pub fn scope_panel_components() -> Scope {
    web::scope("/components")
        .service(product_panel_group::scope())
        .service(edit_product_controller::scope())
        .service(edit_product_variation_controller::scope())
}
