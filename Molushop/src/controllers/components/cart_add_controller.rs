use actix_web::{delete, post, web, http::StatusCode, HttpMessage, HttpRequest, HttpResponse, Scope};
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use serde::Deserialize;
use uuid::Uuid;
use lazy_static::lazy_static;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::middleware::auth::{Auth, SessionData};
use crate::models::components::toast::Toast;
use crate::models::components::nav1_model::CartBadge;
use crate::utils::hx::error_modal_t;
use crate::i18n::I18n;
use crate::models::components::cart_total_v1::CartRoutes;
use crate::constants::urls::CHECKOUT_URL;
use super::scope::SCOPE_COMPONENTS;
use crate::models::error::ServiceError;
use crate::models::models_x::CartProduct;
use crate::services::components::cart_add_service::{self, get_cart_item_count};
use crate::services::components::cart_get_service::get_cart_total_body_render;
use crate::services::components::cart_update_service;

static SCOPE: &str = "/cart";

lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza

    pub static ref ROUTES: CartRoutes = CartRoutes{
        add: format!("{}{}/add", SCOPE_COMPONENTS, SCOPE),
        update: format!("{}{}/update", SCOPE_COMPONENTS, SCOPE),
        remove: format!("{}{}/remove", SCOPE_COMPONENTS, SCOPE),
        restore: format!("{}{}/restore", SCOPE_COMPONENTS, SCOPE),
        checkout: CHECKOUT_URL.to_string()
    };

}

pub fn scope() -> Scope<impl ServiceFactory<ServiceRequest, Config = (), Response = ServiceResponse, Error = actix_web::Error, InitError = ()>> {
    web::scope(SCOPE)
        .wrap(Auth::new())
        .configure(config)
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(add_to_cart);
    cfg.service(update_cart_item);
    cfg.service(remove_cart_item);
    cfg.service(restore_cart_item);
}

#[derive(Deserialize)]
pub struct AddToCartForm {
    pub product_var_id: Uuid,
    pub quantity: i32,
}

#[post("/add")]
async fn add_to_cart(
    i18n: I18n,
    form: web::Form<AddToCartForm>,
    pool_data: web::Data<DbPool>,
    req: HttpRequest,
) -> HttpResponse {
    // Verificar autenticación
    let session = req.extensions().get::<SessionData>().cloned();
    let user_id = match session {
        Some(s) => s.id,
        None => {
            return HttpResponse::Unauthorized()
                .insert_header(("HX-Trigger", "auth-required"))
                .body("No autenticado");
        }
    };

    let form_data = form.into_inner();
    let pool = pool_data.get_ref();

    if form_data.quantity <= 0 {
        return error_modal_t(StatusCode::BAD_REQUEST, &i18n, "product-quantity-positive");
    }

    match cart_add_service::add_to_cart(&user_id, &form_data.product_var_id, form_data.quantity, pool).await {
        Ok(_) => {
            let cart_count = get_cart_item_count(&user_id, pool).await;
            // Toast y no modal: ha salido bien y el número del carrito ya sube
            // (con bote). Los dos por OOB: el botón de la ficha va con
            // hx-swap="none".
            let toast = Toast::translated(&i18n, "cart-added")
                .with_link(i18n.url("/cart"), i18n.t("cart-view"));
            HttpResponse::Ok()
                .body(toast.oob() + &CartBadge::new(cart_count).bumped().oob())
        }
        Err(ServiceError::VariationNotFound) => {
            error_modal_t(StatusCode::BAD_REQUEST, &i18n, "cart-error-variation")
        }
        Err(ServiceError::NotEnoughStock) => {
            error_modal_t(StatusCode::BAD_REQUEST, &i18n, "cart-error-stock")
        }
        Err(e) => {
            tracing::error!(error = %e, "could not add to the cart");
            error_modal_t(StatusCode::INTERNAL_SERVER_ERROR, &i18n, "cart-error-add")
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateCartForm {
    pub cart_product_id: Uuid,
    pub quantity: i32,
}

/// El carrito re-renderizado y, por OOB, el número del menú. Si se ha borrado
/// una línea (`removed`), también el toast para deshacerlo. El JS del carrito
/// lo pinta con htmx.swap, que reparte lo OOB.
async fn cart_body_response(i18n: &I18n, user_id: &Uuid, pool: &DbPool, removed: Option<&CartProduct>) -> HttpResponse {
    let cart_count = get_cart_item_count(user_id, pool).await;
    let mut body = get_cart_total_body_render(i18n, Some(user_id), pool).await;
    body.push_str(&CartBadge::new(cart_count).oob());
    if let Some(line) = removed {
        body.push_str(&removed_toast(i18n, line).oob());
    }
    HttpResponse::Ok().body(body)
}

/// "Producto eliminado · Deshacer". El botón lleva lo que manda el JS a
/// `/restore` (RestoreCartForm), ya como formulario.
fn removed_toast(i18n: &I18n, line: &CartProduct) -> Toast {
    let restore = format!("product_var_id={}&quantity={}", line.product_var_id, line.quantity);
    Toast::translated(i18n, "cart-removed-toast")
        .with_action(i18n.t("cart-undo"), "cart-restore", restore)
}

fn cart_error_response(i18n: &I18n, e: ServiceError) -> HttpResponse {
    match e {
        ServiceError::CartItemNotFound => {
            error_modal_t(StatusCode::NOT_FOUND, i18n, "cart-error-line-gone")
        }
        ServiceError::VariationNotFound => {
            error_modal_t(StatusCode::BAD_REQUEST, i18n, "cart-error-variation")
        }
        ServiceError::NotEnoughStock => {
            error_modal_t(StatusCode::BAD_REQUEST, i18n, "cart-error-stock")
        }
        e => {
            tracing::error!(error = %e, "could not update the cart");
            error_modal_t(StatusCode::INTERNAL_SERVER_ERROR, i18n, "cart-error-update")
        }
    }
}

fn session_user_id(req: &HttpRequest) -> Option<Uuid> {
    req.extensions().get::<SessionData>().cloned().map(|s| s.id)
}

fn unauthorized() -> HttpResponse {
    HttpResponse::Unauthorized()
        .insert_header(("HX-Trigger", "auth-required"))
        .body("No autenticado")
}

/// Fija la cantidad de una línea. Cantidad <= 0 elimina la línea.
#[post("/update")]
async fn update_cart_item(
    i18n: I18n,
    form: web::Form<UpdateCartForm>,
    pool_data: web::Data<DbPool>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match session_user_id(&req) {
        Some(id) => id,
        None => return unauthorized(),
    };

    let form_data = form.into_inner();
    let pool = pool_data.get_ref();

    match cart_update_service::update_cart_item(&user_id, &form_data.cart_product_id, form_data.quantity, pool).await {
        Ok(removed) => cart_body_response(&i18n, &user_id, pool, removed.as_ref()).await,
        Err(e) => cart_error_response(&i18n, e),
    }
}

#[delete("/remove/{cart_product_id}")]
async fn remove_cart_item(
    i18n: I18n,
    path: web::Path<Uuid>,
    pool_data: web::Data<DbPool>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match session_user_id(&req) {
        Some(id) => id,
        None => return unauthorized(),
    };

    let cart_product_id = path.into_inner();
    let pool = pool_data.get_ref();

    match cart_update_service::remove_cart_item(&user_id, &cart_product_id, pool).await {
        Ok(line) => cart_body_response(&i18n, &user_id, pool, Some(&line)).await,
        Err(e) => cart_error_response(&i18n, e),
    }
}

#[derive(Deserialize)]
pub struct RestoreCartForm {
    pub product_var_id: Uuid,
    pub quantity: i32,
}

/// El "Deshacer" del toast de una línea borrada: la vuelve a añadir con sus
/// unidades. Con el alta de siempre, así que comprueba el stock y entra con el
/// precio de ahora; si mientras tanto se ha vuelto a añadir, se suman.
#[post("/restore")]
async fn restore_cart_item(
    i18n: I18n,
    form: web::Form<RestoreCartForm>,
    pool_data: web::Data<DbPool>,
    req: HttpRequest,
) -> HttpResponse {
    let user_id = match session_user_id(&req) {
        Some(id) => id,
        None => return unauthorized(),
    };

    let form_data = form.into_inner();
    let pool = pool_data.get_ref();

    if form_data.quantity <= 0 {
        return error_modal_t(StatusCode::BAD_REQUEST, &i18n, "product-quantity-positive");
    }

    match cart_add_service::add_to_cart(&user_id, &form_data.product_var_id, form_data.quantity, pool).await {
        Ok(_) => cart_body_response(&i18n, &user_id, pool, None).await,
        Err(e) => cart_error_response(&i18n, e),
    }
}
