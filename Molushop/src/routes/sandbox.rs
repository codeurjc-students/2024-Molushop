//! `/sandbox`: pruebas de UI con el CSS y los datos de verdad, antes de tocar
//! el componente real. Como `/ui`, solo existe en la compilación de desarrollo
//! (`cargo run`); en `--release` no se registra y da 404. Por eso no pide
//! sesión aunque enseñe datos del panel.

use actix_web::{http::StatusCode, web, HttpResponse};
use askama::Template;

use crate::models::pages_models::sandbox_page_model::SandboxPage;
use crate::routes::panel_products::html;

pub fn config(cfg: &mut web::ServiceConfig) {
    if cfg!(debug_assertions) {
        cfg.route("/sandbox", web::get().to(sandbox_index));
    }
}

async fn sandbox_index() -> HttpResponse {
    let page = SandboxPage {
        user_logged: false,
        page_name: "Sandbox".to_string(),
    }.render().unwrap();
    html(StatusCode::OK, page)
}
