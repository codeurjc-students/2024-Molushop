use actix_web::{web, HttpResponse, http::Method};
use crate::i18n::I18n;
use uuid::Uuid;
use askama::Template;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::models::pages_models::not_found_page_model::NotFoundPage;
use crate::services::components::{nav1_service::get_nav1_object, login_base_service};
use crate::services::user_service::get_user_by_id;
use crate::middleware::auth::SessionData;

/// La página 404 de la tienda, con la barra y la sesión del usuario. La usan
/// la ruta por defecto (cualquier URL que no exista) y las páginas de algo que
/// no está a la venta, como la ficha de un producto. Los textos son claves de
/// `locales/*/shop.ftl`.
pub async fn not_found_page(i18n: I18n, page_name: &str, title: &str, message: &str, user_id: Option<&Uuid>, pool: &DbPool) -> HttpResponse {
    let mut user_name = "".to_string();
    let mut user_logged = false;
    if let Some(user_id) = user_id {
        if let Ok(user) = get_user_by_id(user_id, pool).await {
            user_name = user.username;
            user_logged = true;
        }
    }

    let (page_name, title, message) = (i18n.t(page_name), i18n.t(title), i18n.t(message));
    let render = NotFoundPage{
        user_logged,
        page_name,
        nav1: get_nav1_object(user_name, user_id, pool).await,
        login_base_data: login_base_service::get_login_base_model_data(),
        i18n: i18n,
        title,
        message
    }.render().unwrap();

    HttpResponse::NotFound().content_type("text/html; charset=utf-8").body(render)
}

/// Ruta por defecto (main.rs): ninguna otra encaja. Solo una página pedida con
/// GET lleva la 404 de la tienda; un fragmento de htmx, un POST o un estático
/// que no existe (CSS, imagen...) se quedan en un 404 sin cuerpo. El boost pide
/// páginas con HX-Request, pero también con HX-Boosted.
pub async fn default_not_found(i18n: I18n, req: actix_web::HttpRequest, pool_data: web::Data<DbPool>, opt_session_data: Option<web::ReqData<SessionData>>) -> HttpResponse {
    let is_fragment = req.headers().contains_key("HX-Request") && !req.headers().contains_key("HX-Boosted");
    let is_static = ["/static/", "/assets/", "/css/"].iter().any(|prefix| req.path().starts_with(prefix));
    if req.method() != Method::GET || is_fragment || is_static {
        return HttpResponse::NotFound().finish();
    }

    let user_id = opt_session_data.map(|session_data| session_data.into_inner().id);
    not_found_page(
        i18n,
        "notfound-page-title",
        "notfound-title",
        "notfound-text",
        user_id.as_ref(),
        pool_data.get_ref()
    ).await
}
