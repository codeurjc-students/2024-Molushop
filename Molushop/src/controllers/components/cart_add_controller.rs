use actix_web::{post, web, HttpMessage, HttpRequest, HttpResponse, Scope};
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use serde::Deserialize;
use uuid::Uuid;
use askama::Template;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::middleware::auth::{Auth, SessionData};
use crate::models::components::modal_1_model::Modal1;
use crate::models::error::ServiceError;
use crate::services::components::cart_add_service;

static SCOPE: &str = "/cart";

pub fn scope() -> Scope<impl ServiceFactory<ServiceRequest, Config = (), Response = ServiceResponse, Error = actix_web::Error, InitError = ()>> {
    web::scope(SCOPE)
        .wrap(Auth::new())
        .configure(config)
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(add_to_cart);
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
            let modal_render = Modal1::new(&msg).render().unwrap();
            HttpResponse::Ok().body(modal_render)
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
