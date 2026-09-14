use actix_web::{web, HttpResponse, HttpRequest};
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;
use askama::Template;

use crate::models::pages_models::order_page_model::*;
use crate::models::pages_models::orders_page_model::*;
use crate::services::components::nav1_service::*;
use crate::services::components::order_get_service::{get_order_object, get_orders_list_object};
use crate::middleware::auth::{Auth, SessionData};
use crate::services::components::login_base_service;
use crate::services::servicesX::get_user_2;
use serde::Deserialize;

/// Query de la ficha. `nuevo=1` lo pone la redirección del checkout al confirmar.
#[derive(Deserialize)]
pub struct OrderDetailQuery {
    nuevo: Option<String>,
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        .route(
            // "Mis pedidos". Sin ella un pedido solo era accesible por su URL:
            // si se perdía, no había forma de volver.
            "/orders",
            // Sin el wrap, opt_session_data llega None y la lista sale vacía
            web::get().to(get_orders).wrap(Auth::new())
        )
        .route(
            // A donde redirige el POST de confirmación, y a donde lleva cada
            // tarjeta de la lista.
            "/orders/{order_id}",
            // Sin el wrap, opt_session_data llega None y no se puede comprobar
            // que el pedido es de quien lo pide
            web::get().to(get_order).wrap(Auth::new())
        )
        ;
}

async fn get_orders(
    pool_data: web::Data<DbPool>,
    opt_session_data: Option<web::ReqData<SessionData>>
) -> HttpResponse {
    let pool = pool_data.get_ref();

    let user_id_opt: Option<Uuid> = opt_session_data.map(|d| d.into_inner().id);

    let mut nombre_aux = "".to_string();
    let mut user_logged = false;

    if let Some(user_id) = user_id_opt.as_ref() {
        if let Ok(user) = get_user_2(user_id, pool).await {
            nombre_aux = user.username;
            user_logged = true;
        }
    }

    let orders_render = OrdersPage{
        user_logged,
        page_name:"Mis pedidos".to_string(),
        nav1:get_nav1_object(nombre_aux, user_id_opt.as_ref(), pool).await,
        login_base_data:login_base_service::get_login_base_model_data(),
        order_list:get_orders_list_object(user_id_opt.as_ref(), pool).await
    }.render().unwrap();

    HttpResponse::Ok().body(orders_render)
}

async fn get_order(
    path: web::Path<Uuid>,
    query: web::Query<OrderDetailQuery>,
    pool_data: web::Data<DbPool>,
    req: HttpRequest,
    opt_session_data: Option<web::ReqData<SessionData>>
) -> HttpResponse {
    let pool = pool_data.get_ref();
    let order_id = path.into_inner();

    let user_id_opt: Option<Uuid> = opt_session_data.map(|d| d.into_inner().id);

    let mut nombre_aux = "".to_string();
    let mut user_logged = false;

    if let Some(user_id) = user_id_opt.as_ref() {
        if let Ok(user) = get_user_2(user_id, pool).await {
            nombre_aux = user.username;
            user_logged = true;
        }
    }

    let mut order = get_order_object(user_id_opt.as_ref(), &order_id, pool).await;
    // Solo un pedido encontrado puede ser "nuevo": con ?nuevo=1 en la URL de un pedido
    // ajeno o inexistente, la ficha sigue diciendo que no existe.
    order.is_new = order.found && query.nuevo.as_deref() == Some("1");

    let order_render = OrderPage{
        user_logged,
        page_name:"Pedido".to_string(),
        nav1:get_nav1_object(nombre_aux, user_id_opt.as_ref(), pool).await,
        login_base_data:login_base_service::get_login_base_model_data(),
        order
    }.render().unwrap();

    HttpResponse::Ok().body(order_render)
}
