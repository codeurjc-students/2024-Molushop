//! `/sandbox`: pruebas de UI con el CSS y los datos de verdad, antes de tocar
//! el componente real. Como `/ui`, solo existe en la compilación de desarrollo
//! (`cargo run`); en `--release` no se registra y da 404. Por eso no pide
//! sesión aunque enseñe datos del panel.

use actix_web::{http::StatusCode, web, HttpResponse};
use askama::Template;
use serde::Deserialize;
use uuid::Uuid;

use diesel::sql_query;
use diesel::sql_types::{Nullable, Text, Uuid as SqlUuid};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use diesel_async::RunQueryDsl;
type DbPool = Pool<AsyncPgConnection>;

use crate::models::pages_models::sandbox_page_model::SandboxPage;
use crate::models::pages_models::sandbox_variations_page_model::{SandboxVariationsPage, VariationsDemo};
use crate::routes::panel_products::html;
use crate::services::products_panel;

pub fn config(cfg: &mut web::ServiceConfig) {
    if cfg!(debug_assertions) {
        cfg.route("/sandbox", web::get().to(sandbox_index))
            .route("/sandbox/variaciones", web::get().to(sandbox_variations));
    }
}

async fn sandbox_index() -> HttpResponse {
    let page = SandboxPage {
        user_logged: false,
        page_name: "Sandbox".to_string(),
    }.render().unwrap();
    html(StatusCode::OK, page)
}

#[derive(Deserialize)]
struct StoreQuery {
    store: Option<Uuid>,
}

#[derive(diesel::QueryableByName)]
struct SandboxStore {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    store_name: String,
}

/// `?store=<id>` para elegir tienda; sin él, la que más productos tiene.
async fn sandbox_variations(query: web::Query<StoreQuery>, pool_data: web::Data<DbPool>) -> HttpResponse {
    let pool = pool_data.get_ref();
    let mut conn = pool.get().await.unwrap();
    let store = sql_query(
        "SELECT s.id, s.store_name FROM seller s
         WHERE $1::uuid IS NULL OR s.id = $1
         ORDER BY (SELECT COUNT(*) FROM product_seller ps WHERE ps.seller_id = s.id) DESC
         LIMIT 1"
    )
        .bind::<Nullable<SqlUuid>, _>(query.store)
        .get_result::<SandboxStore>(&mut conn)
        .await;
    drop(conn);

    let store = match store {
        Ok(store) => store,
        Err(diesel::result::Error::NotFound) => return html(StatusCode::NOT_FOUND, "Tienda no encontrada".to_string()),
        Err(e) => {
            println!("Sandbox: {}", e);
            return html(StatusCode::INTERNAL_SERVER_ERROR, "Error cargando la tienda".to_string());
        }
    };

    match products_panel::get_products(&store.id, pool).await {
        Ok(products) => {
            let page = SandboxVariationsPage {
                user_logged: false,
                page_name: "Sandbox · Variaciones".to_string(),
                store_name: store.store_name,
                products: products.into_iter().map(|product| VariationsDemo { product }).collect(),
            }.render().unwrap();
            html(StatusCode::OK, page)
        }
        Err(e) => {
            println!("Sandbox: {}", e);
            html(StatusCode::INTERNAL_SERVER_ERROR, "Error cargando los productos".to_string())
        }
    }
}
