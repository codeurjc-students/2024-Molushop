//! Grupo de tarjetas de la portada y de la búsqueda: "Cargar más productos"
//! pide aquí la tanda siguiente, con el mismo filtro. Público, como ellas
//! (la sesión, si la hay, solo sirve para llenar el corazón de favoritos).

use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::{get, web, HttpMessage, HttpRequest, HttpResponse, Scope};
use askama::Template;
use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Pool;
use lazy_static::lazy_static;
use serde::Deserialize;

use super::scope::SCOPE_COMPONENTS;
use crate::i18n::I18n;
use crate::middleware::auth::{Auth, SessionData};
use crate::models::components::product_card_group_model::{CardFilter, ProductCardGroupPage};
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

/// `Auth` no rechaza a nadie: solo mete la `SessionData` si la cookie vale.
pub fn scope() -> Scope<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    web::scope(SCOPE).wrap(Auth::new()).service(page)
}

/// La tanda y el filtro de la búsqueda (`CardFilter`), si lo hay.
#[derive(Deserialize)]
pub struct PageQuery {
    pub page: Option<i64>,
    pub q: Option<String>,
    pub category: Option<String>,
}

#[get("/page")]
async fn page(i18n: I18n, query: web::Query<PageQuery>, pool_data: web::Data<DbPool>, req: HttpRequest) -> HttpResponse {
    let query = query.into_inner();
    let filter = CardFilter::new(query.q, query.category);
    let user_id = req.extensions().get::<SessionData>().map(|s| s.id);
    let (products, more_url) = get_product_card_group_page(query.page.unwrap_or(2), &filter, user_id.as_ref(), pool_data.get_ref()).await;
    HttpResponse::Ok().body(ProductCardGroupPage { products, more_url, i18n: &i18n }.render().unwrap())
}
