use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::http::StatusCode;
use actix_web::{post, web, HttpMessage, HttpRequest, HttpResponse, Scope};
use askama::Template;
use lazy_static::lazy_static;
use uuid::Uuid;

use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Pool;
type DbPool = Pool<AsyncPgConnection>;

use super::scope::SCOPE_COMPONENTS;
use crate::i18n::I18n;
use crate::middleware::auth::{Auth, SessionData};
use crate::models::components::favorite_button::{FavoriteButton, FavoriteButtonBody, FavoritesRoutes};
use crate::models::components::toast::Toast;
use crate::models::error::ServiceError;
use crate::services::favorites_service::{add_favorite, remove_favorite};
use crate::utils::hx::error_modal_t;

pub static SCOPE: &str = "/favorites";

lazy_static! {
    pub static ref ROUTES: FavoritesRoutes = FavoritesRoutes {
        add: format!("{}{}/add", SCOPE_COMPONENTS, SCOPE),
        remove: format!("{}{}/remove", SCOPE_COMPONENTS, SCOPE),
    };
}

/// `Auth` no rechaza a nadie: solo mete la `SessionData` si la cookie vale. Sin
/// ella, los dos POST responden 401 (ver `user_or_401`).
pub fn scope() -> Scope<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    web::scope(SCOPE).wrap(Auth::new()).configure(config)
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(add);
    cfg.service(remove);
}

/// Mismo 401 que el carrito y las opiniones: el `HX-Trigger` lo recoge
/// login_modal.js y abre el login; el corazón se queda como estaba.
fn user_or_401(req: &HttpRequest) -> Result<Uuid, HttpResponse> {
    req.extensions()
        .get::<SessionData>()
        .map(|s| s.id)
        .ok_or_else(|| {
            HttpResponse::Unauthorized()
                .insert_header(("HX-Trigger", "auth-required"))
                .body("No autenticado")
        })
}

/// El botón ya cambiado, que sustituye al pulsado (`hx-swap="outerHTML"`), y
/// detrás un toast. Al guardar, el corazón late y el toast lleva a la página
/// de favoritos, salvo si ya se está en ella.
fn button(i18n: &I18n, req: &HttpRequest, product_id: Uuid, active: bool) -> HttpResponse {
    let fav = FavoriteButton::new(product_id, active);
    let (fav, toast) = if active {
        let mut toast = Toast::translated(i18n, "favorites-added-toast");
        if !on_favorites_page(req) {
            toast = toast.with_link(i18n.url("/favorites"), i18n.t("favorites-view"));
        }
        (fav.popped(), toast)
    } else {
        (fav, Toast::translated(i18n, "favorites-removed-toast"))
    };
    let body = FavoriteButtonBody { fav, i18n }.render().unwrap();
    HttpResponse::Ok().body(body + &toast.oob())
}

/// La página desde la que se pulsa (htmx manda su URL en `HX-Current-URL`).
fn on_favorites_page(req: &HttpRequest) -> bool {
    req.headers()
        .get("HX-Current-URL")
        .and_then(|v| v.to_str().ok())
        .and_then(|url| url.split('?').next())
        .is_some_and(|url| url.ends_with("/favorites"))
}

#[post("/add/{product_id}")]
async fn add(
    i18n: I18n,
    path: web::Path<Uuid>,
    pool_data: web::Data<DbPool>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match user_or_401(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let product_id = path.into_inner();

    match add_favorite(&user_id, &product_id, pool_data.get_ref()).await {
        Ok(()) => button(&i18n, &req, product_id, true),
        // Se ha retirado de la venta con la ficha abierta (o el POST es a mano)
        Err(ServiceError::ProductNotFound) => {
            error_modal_t(StatusCode::NOT_FOUND, &i18n, "favorites-error-gone")
        }
        Err(e) => {
            tracing::error!(error = %e, "could not add the favorite");
            error_modal_t(StatusCode::INTERNAL_SERVER_ERROR, &i18n, "favorites-error")
        }
    }
}

#[post("/remove/{product_id}")]
async fn remove(
    i18n: I18n,
    path: web::Path<Uuid>,
    pool_data: web::Data<DbPool>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match user_or_401(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let product_id = path.into_inner();

    match remove_favorite(&user_id, &product_id, pool_data.get_ref()).await {
        Ok(()) => button(&i18n, &req, product_id, false),
        Err(e) => {
            tracing::error!(error = %e, "could not remove the favorite");
            error_modal_t(StatusCode::INTERNAL_SERVER_ERROR, &i18n, "favorites-error")
        }
    }
}
