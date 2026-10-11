//! Pedidos de una tienda en el panel: `/panel/stores/{store_id}/orders` y
//! `.../orders/{order_id}`. Se registran dentro del scope `/panel`, así que
//! `PanelGuard` ya ha comprobado el rol; aquí se comprueba la tienda de la URL.
//!
//! La tienda solo ve y cambia SUS líneas de cada pedido. El cambio de estado es
//! un formulario HTML normal: el POST redirige (303) a la ficha con `?ok=` o
//! `?error=`, para que recargar no lo repita.

use actix_web::{http::header, http::StatusCode, web, HttpResponse};
use askama::Template;
use serde::Deserialize;
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::models::components::panel_nav::PanelNav;
use crate::models::pages_models::panel_orders_page_model::{PanelOrderPage, PanelOrdersPage};
use crate::routes::panel_products::{html, store_or_response};
use crate::services::panel_orders_service::{get_order, list_orders, set_line_status, PanelOrderError};
use crate::services::panel_role_service::PanelUser;

/// Rutas relativas al scope `/panel`.
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/stores/{store_id}/orders", web::get().to(orders_list))
        .route("/stores/{store_id}/orders/{order_id}", web::get().to(order_detail))
        .route(
            "/stores/{store_id}/orders/{order_id}/lines/{item_id}/status",
            web::post().to(line_status),
        );
}

#[derive(Deserialize)]
pub struct OrderQuery {
    pub ok: Option<String>,
    pub error: Option<String>,
}

// Los avisos viajan como códigos y el texto se pone aquí: así la URL no puede
// hacer que la página diga lo que alguien quiera.
pub fn ok_message(code: &str) -> Option<&'static str> {
    match code {
        "status" => Some("Estado de la línea actualizado."),
        _ => None,
    }
}

pub fn error_message(code: &str) -> Option<&'static str> {
    match code {
        "transition" => Some("Ese cambio de estado no es posible (puede que la línea ya haya cambiado)."),
        "db" => Some("Ha ocurrido un error. No se ha guardado nada."),
        _ => None,
    }
}

async fn orders_list(
    path: web::Path<Uuid>,
    panel_user: web::ReqData<PanelUser>,
    pool_data: web::Data<DbPool>,
) -> HttpResponse {
    let pool = pool_data.get_ref();
    let store = match store_or_response(&panel_user, &path, pool).await {
        Ok(store) => store,
        Err(resp) => return resp,
    };

    match list_orders(Some(&store.id), pool).await {
        Ok(orders) => {
            let page = PanelOrdersPage {
                user_logged: true,
                page_name: format!("Pedidos - {}", store.store_name),
                panel_nav: PanelNav::new(&panel_user, "orders").with_store(&store),
                title: format!("Pedidos de {}", store.store_name),
                base_url: format!("/panel/stores/{}/orders", store.id),
                all_lines: false,
                orders,
            }.render().unwrap();
            html(StatusCode::OK, page)
        }
        Err(e) => {
            println!("Panel: error cargando los pedidos de la tienda {}: {:?}", store.id, e);
            html(StatusCode::INTERNAL_SERVER_ERROR, "Error cargando los pedidos".to_string())
        }
    }
}

/// 404 tanto si el pedido no existe como si no lleva nada de esta tienda.
async fn order_detail(
    path: web::Path<(Uuid, Uuid)>,
    query: web::Query<OrderQuery>,
    panel_user: web::ReqData<PanelUser>,
    pool_data: web::Data<DbPool>,
) -> HttpResponse {
    let (store_id, order_id) = path.into_inner();
    let pool = pool_data.get_ref();
    let store = match store_or_response(&panel_user, &store_id, pool).await {
        Ok(store) => store,
        Err(resp) => return resp,
    };

    match get_order(Some(&store.id), &order_id, pool).await {
        Ok(Some(order)) => {
            let query = query.into_inner();
            let base_url = format!("/panel/stores/{}/orders", store.id);
            let page = PanelOrderPage {
                user_logged: true,
                page_name: format!("Pedido nº {} - {}", order.order_number, store.store_name),
                panel_nav: PanelNav::new(&panel_user, "orders").with_store(&store),
                base_url,
                all_lines: false,
                order,
                ok_message: query.ok.as_deref().and_then(ok_message),
                error_message: query.error.as_deref().and_then(error_message),
            }.render().unwrap();
            html(StatusCode::OK, page)
        }
        Ok(None) => html(StatusCode::NOT_FOUND, "Pedido no encontrado".to_string()),
        Err(e) => {
            println!("Panel: error cargando el pedido {}: {:?}", order_id, e);
            html(StatusCode::INTERNAL_SERVER_ERROR, "Error cargando el pedido".to_string())
        }
    }
}

#[derive(Deserialize)]
pub struct LineStatusForm {
    pub status: i16,
}

async fn line_status(
    path: web::Path<(Uuid, Uuid, Uuid)>,
    form: web::Form<LineStatusForm>,
    panel_user: web::ReqData<PanelUser>,
    pool_data: web::Data<DbPool>,
) -> HttpResponse {
    let (store_id, order_id, item_id) = path.into_inner();
    let pool = pool_data.get_ref();
    let store = match store_or_response(&panel_user, &store_id, pool).await {
        Ok(store) => store,
        Err(resp) => return resp,
    };

    let result = set_line_status(Some(&store.id), &order_id, &item_id, form.status, pool).await;
    back(&format!("/panel/stores/{}/orders/{}", store.id, order_id), result)
}

/// Vuelve a la ficha del pedido con el aviso del resultado.
pub fn back(order_url: &str, result: Result<(), PanelOrderError>) -> HttpResponse {
    let location = match result {
        Ok(()) => format!("{}?ok=status", order_url),
        Err(e) => {
            if let PanelOrderError::Db(detail) = &e {
                println!("Panel pedidos: {}", detail);
            }
            format!("{}?error={}", order_url, e.code())
        }
    };
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, location))
        .finish()
}
