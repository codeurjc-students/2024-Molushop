use actix_web::{web, HttpResponse};
use askama::Template;

use crate::models::pages_models::ui_catalog_page_model::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .route(
            // Sin sesión ni base de datos: es una página de referencia para
            // quien desarrolla, no enlazada desde la tienda.
            "/ui",
            web::get().to(get_ui_catalog)
        )
        ;
}

async fn get_ui_catalog() -> HttpResponse {
    let ui_catalog_render = UiCatalogPage{
        user_logged: false,
        page_name:"Catálogo de UI".to_string()
    }.render().unwrap();

    HttpResponse::Ok().body(ui_catalog_render)
}
