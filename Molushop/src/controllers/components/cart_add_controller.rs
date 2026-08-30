use actix_web::{delete, post, web, HttpMessage, HttpRequest, HttpResponse, Scope};
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use serde::Deserialize;
use uuid::Uuid;
use askama::Template;
use lazy_static::lazy_static;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::middleware::auth::{Auth, SessionData};
use crate::models::components::modal_1_model::Modal1;
use crate::models::components::cart_total_v1::CartRoutes;
use super::scope::SCOPE_COMPONENTS;
use crate::models::error::ServiceError;
use crate::services::components::cart_add_service::{self, get_cart_item_count};
use crate::services::components::cart_get_service::get_cart_total_body_render;
use crate::services::components::cart_update_service;

static SCOPE: &str = "/cart";

lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza

    pub static ref ROUTES: CartRoutes = CartRoutes{
        add: format!("{}{}/add", SCOPE_COMPONENTS, SCOPE),
        update: format!("{}{}/update", SCOPE_COMPONENTS, SCOPE),
        remove: format!("{}{}/remove", SCOPE_COMPONENTS, SCOPE)
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
}

#[derive(Deserialize)]
pub struct AddToCartForm {
    pub product_var_id: Uuid,
    pub quantity: i32,
}

#[post("/add")]
async fn add_to_cart(
    form: web::Json<AddToCartForm>,
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
        let modal_render = Modal1::new("La cantidad debe ser mayor a 0").render().unwrap();
        return HttpResponse::BadRequest().body(modal_render);
    }

    match cart_add_service::add_to_cart(&user_id, &form_data.product_var_id, form_data.quantity, pool).await {
        Ok(msg) => {
            let cart_count = get_cart_item_count(&user_id, pool).await;
            let modal_render = Modal1::new(&msg).render().unwrap();
            HttpResponse::Ok()
                .insert_header(("X-Cart-Count", cart_count.to_string()))
                .body(modal_render)
        }
        Err(ServiceError::VariationNotFound) => {
            let modal_render = Modal1::new("Variación no encontrada").render().unwrap();
            HttpResponse::BadRequest().body(modal_render)
        }
        Err(ServiceError::NotEnoughStock) => {
            let modal_render = Modal1::new("No hay suficiente stock").render().unwrap();
            HttpResponse::BadRequest().body(modal_render)
        }
        Err(e) => {
            println!("Error al añadir al carrito: {}", e);
            let modal_render = Modal1::new("Error al añadir al carrito").render().unwrap();
            HttpResponse::InternalServerError().body(modal_render)
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateCartForm {
    pub cart_product_id: Uuid,
    pub quantity: i32,
}

/// Devuelve el cuerpo del carrito re-renderizado junto al nuevo contador,
/// que es lo que consumen tanto el componente como el badge del nav.
async fn cart_body_response(user_id: &Uuid, pool: &DbPool) -> HttpResponse {
    let cart_count = get_cart_item_count(user_id, pool).await;
    let body = get_cart_total_body_render(Some(user_id), pool).await;
    HttpResponse::Ok()
        .insert_header(("X-Cart-Count", cart_count.to_string()))
        .body(body)
}

fn cart_error_response(e: ServiceError) -> HttpResponse {
    match e {
        ServiceError::CartItemNotFound => {
            let modal_render = Modal1::new("El producto ya no está en tu cesta").render().unwrap();
            HttpResponse::NotFound().body(modal_render)
        }
        ServiceError::VariationNotFound => {
            let modal_render = Modal1::new("Variación no encontrada").render().unwrap();
            HttpResponse::BadRequest().body(modal_render)
        }
        ServiceError::NotEnoughStock => {
            let modal_render = Modal1::new("No hay suficiente stock").render().unwrap();
            HttpResponse::BadRequest().body(modal_render)
        }
        e => {
            println!("Error al modificar el carrito: {}", e);
            let modal_render = Modal1::new("Error al modificar el carrito").render().unwrap();
            HttpResponse::InternalServerError().body(modal_render)
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
    form: web::Json<UpdateCartForm>,
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
        Ok(_) => cart_body_response(&user_id, pool).await,
        Err(e) => cart_error_response(e),
    }
}

#[delete("/remove/{cart_product_id}")]
async fn remove_cart_item(
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
        Ok(_) => cart_body_response(&user_id, pool).await,
        Err(e) => cart_error_response(e),
    }
}
