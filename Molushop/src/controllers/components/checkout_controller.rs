use actix_web::{post, web, HttpMessage, HttpRequest, HttpResponse, Scope};
use crate::i18n::I18n;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use uuid::Uuid;
use lazy_static::lazy_static;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::middleware::auth::{Auth, SessionData};
use crate::models::components::checkout_v1::{CheckoutError, CheckoutRoutes, ConfirmForm};
use crate::models::error::ServiceError;
use crate::constants::urls::{CART_URL, ORDER_URL_PREFIX};
use super::scope::SCOPE_COMPONENTS;
use crate::services::components::checkout_confirm_service::confirm_order;
use crate::services::components::checkout_get_service::get_checkout_object;
use crate::routes::checkout::render_checkout_page;

static SCOPE: &str = "/checkout";

lazy_static! {

    /// Las rutas del checkout se construyen aquí, que es donde el resto del
    /// proyecto las construye. Vivieron un tiempo en `models/components/
    /// checkout_v1.rs` porque el frontend se escribió antes que este
    /// controlador.
    pub static ref ROUTES: CheckoutRoutes = CheckoutRoutes{
        confirm: format!("{}{}/confirm", SCOPE_COMPONENTS, SCOPE),
        cart: CART_URL.to_string()
    };

}

pub fn scope() -> Scope<impl ServiceFactory<ServiceRequest, Config = (), Response = ServiceResponse, Error = actix_web::Error, InitError = ()>> {
    web::scope(SCOPE)
        .wrap(Auth::new())
        .configure(config)
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(confirm);
}

/// Confirma el pedido.
///
/// Recibe `web::Form` y no `web::Json` a propósito: el checkout es un formulario
/// de HTML normal para que funcione sin JS.
///
/// - Éxito: `303 See Other` a la página del pedido. Con un POST de formulario,
///   redirigir es lo que evita que un F5 cree un segundo pedido.
/// - Fallo: se repinta ESTA misma página con el error y la dirección tecleada.
///   Redirigir con un `?error=` la perdería, y `customer_address` está vacía,
///   así que no habría de dónde recuperarla.
#[post("/confirm")]
async fn confirm(
    i18n: I18n,
    form: web::Form<ConfirmForm>,
    pool_data: web::Data<DbPool>,
    req: HttpRequest,
) -> HttpResponse {
    let session = req.extensions().get::<SessionData>().cloned();
    let user_id = match session {
        Some(s) => s.id,
        None => {
            return HttpResponse::Found()
                .insert_header(("Location", i18n.url(CART_URL)))
                .finish();
        }
    };

    let form = form.into_inner();
    let pool = pool_data.get_ref();

    // La validación de la dirección es del servidor aunque los inputs sean
    // `required`: el `required` del navegador no es una garantía.
    if !form.address_is_complete() {
        let message = i18n.t("checkout-error-address");
        return repaint(i18n, &user_id, &form, &message, None, pool).await;
    }

    match confirm_order(&user_id, &form, pool).await {
        Ok(order) => HttpResponse::SeeOther()
            // ?nuevo=1: la ficha solo dice "¡Pedido confirmado!" al llegar desde aquí
            // (lo lee get_order en routes/orders.rs)
            .insert_header(("Location", i18n.url(&format!("{}{}?nuevo=1", ORDER_URL_PREFIX, order.id))))
            .finish(),

        Err(ServiceError::NotEnoughStockLine { line_id, product_name, available }) => {
            let message = if available > 0 {
                i18n.t_args("checkout-error-stock-low", &[("product", product_name.into()), ("available", (available as i64).into())])
            } else {
                i18n.t1("checkout-error-stock-out", "product", &product_name)
            };
            repaint(i18n, &user_id, &form, &message, Some(line_id), pool).await
        }

        Err(ServiceError::EmptyCart) => {
            // Sin líneas no hay página que repintar con sentido: el carrito es
            // el sitio al que mandarlo.
            HttpResponse::SeeOther()
                .insert_header(("Location", i18n.url(CART_URL)))
                .finish()
        }

        Err(ServiceError::ProductUnavailable { line_id, product_name }) => {
            let message = i18n.t1("checkout-error-unavailable", "product", &product_name);
            repaint(i18n, &user_id, &form, &message, Some(line_id), pool).await
        }

        Err(ServiceError::MissingPrice) => {
            let message = i18n.t("checkout-error-price");
            repaint(i18n, &user_id, &form, &message, None, pool).await
        }

        Err(e) => {
            tracing::error!(error = %e, "could not place the order");
            let message = i18n.t("checkout-error");
            repaint(i18n, &user_id, &form, &message, None, pool).await
        }
    }
}

/// Repinta el checkout con el error, conservando lo que el usuario había escrito.
/// `409 Conflict` y no `200`: la petición era válida pero el estado del catálogo
/// no permitió cumplirla.
async fn repaint(
    i18n: I18n,
    user_id: &Uuid,
    form: &ConfirmForm,
    message: &str,
    line_id: Option<Uuid>,
    pool: &DbPool,
) -> HttpResponse {
    let mut checkout = get_checkout_object(Some(user_id), Some(&form.lines), pool).await;

    // La dirección que se repinta es la que acaba de teclear, no la de la BD
    checkout.address = form.address();
    checkout.error = Some(CheckoutError {
        message: message.to_string(),
        line_id,
    });

    HttpResponse::Conflict()
        .content_type("text/html; charset=utf-8")
        .body(render_checkout_page(i18n, Some(user_id), checkout, pool).await)
}
