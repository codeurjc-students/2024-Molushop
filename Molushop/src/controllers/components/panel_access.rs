//! Comprobación de propiedad para los endpoints de componentes del panel.
//!
//! Todos cuelgan del scope `/panel`, así que `PanelGuard` ya ha dejado un
//! `PanelUser`; aquí solo se mira si el producto/variación es suyo. Ajeno o
//! inexistente -> 404 (no se revela qué existe), error de BD -> 500. Las dos
//! llevan el aviso con `error_modal`, que htmx sí pinta.

use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::utils::hx::error_modal;
use crate::services::panel_role_service::{can_edit_product, can_edit_variation, PanelUser};

fn to_response(result: Result<bool, String>) -> Result<(), HttpResponse> {
    match result {
        Ok(true) => Ok(()),
        Ok(false) => Err(error_modal(StatusCode::NOT_FOUND, "Producto no encontrado")),
        Err(e) => {
            println!("Panel: {}", e);
            Err(error_modal(StatusCode::INTERNAL_SERVER_ERROR, "Ha ocurrido un error"))
        }
    }
}

/// `Err(respuesta)` si el usuario no puede tocar el producto.
pub async fn product_access(
    panel_user: &PanelUser,
    product_id: &Uuid,
    pool: &DbPool,
) -> Result<(), HttpResponse> {
    to_response(can_edit_product(panel_user, product_id, pool).await)
}

/// `Err(respuesta)` si el usuario no puede tocar la variación.
pub async fn variation_access(
    panel_user: &PanelUser,
    variation_id: &Uuid,
    pool: &DbPool,
) -> Result<(), HttpResponse> {
    to_response(can_edit_variation(panel_user, variation_id, pool).await)
}
