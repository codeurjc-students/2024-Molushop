use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, http::Method};
use crate::i18n::I18n;
use uuid::Uuid;
use askama::Template;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::models::pages_models::not_found_page_model::NotFoundPage;
use crate::services::components::login_base_service;
use crate::services::components::nav1_service::shop_nav;
use crate::middleware::auth::SessionData;
use crate::middleware::request_log::is_static;

/// La página 404 de la tienda, con la barra y la sesión del usuario. La usan
/// la ruta por defecto (cualquier URL que no exista) y las páginas de algo que
/// no está a la venta, como la ficha de un producto. Los textos son claves de
/// `locales/*/shop.ftl`.
pub async fn not_found_page(i18n: I18n, page_name: &str, title: &str, message: &str, user_id: Option<&Uuid>, pool: &DbPool) -> HttpResponse {
    let (user_logged, nav1) = shop_nav(user_id, pool).await;

    let (page_name, title, message) = (i18n.t(page_name), i18n.t(title), i18n.t(message));
    let render = NotFoundPage{
        user_logged,
        page_name,
        nav1,
        login_base_data: login_base_service::get_login_base_model_data(),
        i18n: i18n,
        title,
        message
    }.render().unwrap();

    HttpResponse::NotFound().content_type("text/html; charset=utf-8").body(render)
}

/// Ruta por defecto (main.rs): ninguna otra encaja. Lleva `Auth` para que la
/// barra salga con la sesión.
pub async fn default_not_found(req: HttpRequest) -> HttpResponse {
    not_found_response(&req).await
}

/// La respuesta a algo que no existe: una URL sin ruta o con un trozo que no se
/// puede leer (`middleware::invalid_path`). Solo una página pedida con GET lleva
/// la 404 de la tienda; un fragmento de htmx, un POST o un estático que no
/// existe (CSS, imagen...) se quedan en un 404 sin cuerpo. El boost pide
/// páginas con HX-Request, pero también con HX-Boosted.
/// La sesión la deja `Auth` en la petición si la ruta lo lleva; si no, sale
/// como si no hubiera.
pub async fn not_found_response(req: &HttpRequest) -> HttpResponse {
    let is_fragment = req.headers().contains_key("HX-Request") && !req.headers().contains_key("HX-Boosted");
    if req.method() != Method::GET || is_fragment || is_static(req.path()) {
        return HttpResponse::NotFound().finish();
    }
    let Some(pool_data) = req.app_data::<web::Data<DbPool>>() else {
        return HttpResponse::NotFound().finish();
    };

    let user_id = req.extensions().get::<SessionData>().map(|session_data| session_data.id);
    not_found_page(
        I18n::from_http_request(req),
        "notfound-page-title",
        "notfound-title",
        "notfound-text",
        user_id.as_ref(),
        pool_data.get_ref()
    ).await
}
