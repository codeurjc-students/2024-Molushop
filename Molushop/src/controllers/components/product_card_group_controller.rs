//! Grupo de tarjetas de la portada: "Cargar más productos" pide aquí la
//! tanda siguiente. Público, como la portada.

use actix_web::{get, web, HttpResponse, Scope};
use askama::Template;
use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Pool;
use lazy_static::lazy_static;
use serde::Deserialize;

use super::scope::SCOPE_COMPONENTS;
use crate::models::components::product_card_group_model::ProductCardGroupPage;
use crate::services::components::product_card_group_service::get_product_card_group_page;

type DbPool = Pool<AsyncPgConnection>;

pub static SCOPE: &str = "/product-card-group";

pub struct Routes {
    pub page: String,
}

lazy_static! {
    pub static ref ROUTES: Routes = Routes {
        page: format!("{}{}/page", SCOPE_COMPONENTS, SCOPE),
    };
}

pub fn scope() -> Scope {
    web::scope(SCOPE).service(page)
}

#[derive(Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
}

#[get("/page")]
async fn page(query: web::Query<PageQuery>, pool_data: web::Data<DbPool>) -> HttpResponse {
    let (products, more_url) = get_product_card_group_page(query.page.unwrap_or(2), pool_data.get_ref()).await;
    HttpResponse::Ok().body(ProductCardGroupPage { products, more_url }.render().unwrap())
}
