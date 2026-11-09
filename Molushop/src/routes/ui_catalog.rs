use actix_web::{web, HttpResponse};
use askama::Template;

use crate::models::pages_models::ui_catalog_page_model::*;

/// Sin sesión ni base de datos: es una página de referencia para quien
/// desarrolla, no enlazada desde la tienda. Solo existe en la compilación de
/// desarrollo (`cargo run`); en `--release` no se registra y `/ui` da 404.
pub fn config(cfg: &mut web::ServiceConfig) {
    if cfg!(debug_assertions) {
        cfg.route("/ui", web::get().to(get_ui_catalog));
    }
}

async fn get_ui_catalog() -> HttpResponse {
    let ui_catalog_render = UiCatalogPage{
        user_logged: false,
        page_name:"Catálogo de UI".to_string()
    }.render().unwrap();

    HttpResponse::Ok().body(ui_catalog_render)
}
