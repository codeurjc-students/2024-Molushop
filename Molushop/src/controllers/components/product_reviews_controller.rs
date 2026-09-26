use actix_web::{get, web, HttpResponse, Scope};
use lazy_static::lazy_static;
use serde::Deserialize;
use uuid::Uuid;

use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Pool;
type DbPool = Pool<AsyncPgConnection>;

use super::scope::SCOPE_COMPONENTS;
use crate::models::components::product_reviews_v1::ProductReviewsRoutes;
use crate::services::components::product_reviews_service::get_product_reviews_body_render;

pub static SCOPE: &str = "/product-reviews";

lazy_static! {
    pub static ref ROUTES: ProductReviewsRoutes = ProductReviewsRoutes {
        list: format!("{}{}/list", SCOPE_COMPONENTS, SCOPE),
    };
}

/// Sin `Auth`, al contrario que el carrito: las opiniones se leen sin sesión,
/// igual que la ficha del producto donde salen. El POST de la opinión propia sí
/// la necesitará, y entonces irá envuelto por su cuenta.
pub fn scope() -> Scope {
    web::scope(SCOPE).configure(config)
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(list_reviews);
}

#[derive(Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
}

/// El "ver más": devuelve la sección repintada con una página más de opiniones.
/// Es el cuerpo del componente, sin el wrapper, porque sustituye al que ya está
/// puesto en la ficha.
#[get("/list/{product_id}")]
async fn list_reviews(
    path: web::Path<Uuid>,
    query: web::Query<PageQuery>,
    pool_data: web::Data<DbPool>,
) -> HttpResponse {
    let product_id = path.into_inner();
    let page = query.page.unwrap_or(1);
    let body = get_product_reviews_body_render(&product_id, page, pool_data.get_ref()).await;

    HttpResponse::Ok().body(body)
}
