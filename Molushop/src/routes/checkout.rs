use actix_web::{web, HttpResponse, HttpRequest};
use serde::Deserialize;
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;
use askama::Template;

use crate::models::pages_models::checkout_page_model::*;
use crate::services::components::nav1_service::*;
use crate::services::components::checkout_get_service::get_checkout_object;
use crate::models::components::checkout_v1::CheckoutData;
use crate::middleware::auth::{Auth, SessionData};
use crate::services::components::login_base_service;
use crate::services::servicesX::get_user_2;

/// `?lines=id1,id2`: qué líneas de la cesta se están comprando. Sin él se
/// compra el carrito entero.
#[derive(Deserialize)]
pub struct CheckoutQuery {
    pub lines: Option<String>
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .route(
            "/checkout",
            // Sin el wrap, opt_session_data llega None y la página sale vacía
            web::get().to(get_checkout).wrap(Auth::new())
        )
        ;
}

async fn get_checkout(
    query: web::Query<CheckoutQuery>,
    pool_data: web::Data<DbPool>,
    req: HttpRequest,
    opt_session_data: Option<web::ReqData<SessionData>>
) -> HttpResponse {
    let pool = pool_data.get_ref();

    let user_id_opt: Option<Uuid> = opt_session_data.map(|d| d.into_inner().id);

    let lines = query.into_inner().lines;
    let checkout = get_checkout_object(user_id_opt.as_ref(), lines.as_deref(), pool).await;

    HttpResponse::Ok().body(render_checkout_page(user_id_opt.as_ref(), checkout, pool).await)
}

/// Pinta la página entera alrededor de un `CheckoutData` ya construido.
///
/// Es `pub` porque la usan dos sitios: este GET y el POST de confirmación
/// cuando falla, que repinta esta misma página con el error y la dirección que
/// el usuario acababa de teclear en vez de redirigir y perderla. Sin este
/// helper, el controlador tendría que duplicar el montaje del nav y el modal.
pub async fn render_checkout_page(
    user_id: Option<&Uuid>,
    checkout: CheckoutData,
    pool: &DbPool
) -> String {
    let mut nombre_aux = "".to_string();
    let mut user_logged = false;

    if let Some(user_id) = user_id {
        if let Ok(user) = get_user_2(user_id, pool).await {
            nombre_aux = user.username;
            user_logged = true;
        }
    }

    CheckoutPage{
        user_logged,
        page_name:"Finalizar compra".to_string(),
        nav1:get_nav1_object(nombre_aux, user_id, pool).await,
        login_base_data:login_base_service::get_login_base_model_data(),
        checkout
    }.render().unwrap()
}
