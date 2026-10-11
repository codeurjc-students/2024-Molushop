//! `/admin/orders`: todos los pedidos, con todas sus líneas, solo para el admin.
//! Se registra dentro del scope `/admin`, así que `PanelGuard::admin()` ya ha
//! comprobado el rol.
//!
//! Son las mismas páginas y el mismo servicio que los pedidos de una tienda
//! (`routes/panel_orders.rs`), sin el filtro de tienda: el admin ve y cambia las
//! líneas de cualquiera.

use actix_web::{http::StatusCode, web, HttpResponse};
use askama::Template;
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::models::components::panel_nav::PanelNav;
use crate::models::pages_models::panel_orders_page_model::{PanelOrderPage, PanelOrdersPage};
use crate::routes::panel_orders::{back, error_message, ok_message, LineStatusForm, OrderQuery};
use crate::routes::panel_products::html;
use crate::services::panel_orders_service::{get_order, list_orders, set_line_status};
use crate::services::panel_role_service::PanelUser;

pub static ADMIN_ORDERS: &str = "/admin/orders";

/// Rutas relativas al scope `/admin`.
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/orders", web::get().to(orders_list))
        .route("/orders/{order_id}", web::get().to(order_detail))
        .route("/orders/{order_id}/lines/{item_id}/status", web::post().to(line_status));
}

async fn orders_list(
    panel_user: web::ReqData<PanelUser>,
    pool_data: web::Data<DbPool>,
) -> HttpResponse {
    match list_orders(None, pool_data.get_ref()).await {
        Ok(orders) => {
            let page = PanelOrdersPage {
                user_logged: true,
                page_name: "Panel - Pedidos".to_string(),
                panel_nav: PanelNav::new(&panel_user, "admin_orders"),
                title: "Todos los pedidos".to_string(),
                base_url: ADMIN_ORDERS.to_string(),
                all_lines: true,
                orders,
            }.render().unwrap();
            html(StatusCode::OK, page)
        }
        Err(e) => {
            println!("Admin pedidos: {:?}", e);
            html(StatusCode::INTERNAL_SERVER_ERROR, "Error cargando los pedidos".to_string())
        }
    }
}

async fn order_detail(
    path: web::Path<Uuid>,
    query: web::Query<OrderQuery>,
    panel_user: web::ReqData<PanelUser>,
    pool_data: web::Data<DbPool>,
) -> HttpResponse {
    let order_id = path.into_inner();

    match get_order(None, &order_id, pool_data.get_ref()).await {
        Ok(Some(order)) => {
            let query = query.into_inner();
            let page = PanelOrderPage {
                user_logged: true,
                page_name: format!("Panel - Pedido nº {}", order.order_number),
                panel_nav: PanelNav::new(&panel_user, "admin_orders"),
                base_url: ADMIN_ORDERS.to_string(),
                all_lines: true,
                order,
                ok_message: query.ok.as_deref().and_then(ok_message),
                error_message: query.error.as_deref().and_then(error_message),
            }.render().unwrap();
            html(StatusCode::OK, page)
        }
        Ok(None) => html(StatusCode::NOT_FOUND, "Pedido no encontrado".to_string()),
        Err(e) => {
            println!("Admin pedidos: error cargando el pedido {}: {:?}", order_id, e);
            html(StatusCode::INTERNAL_SERVER_ERROR, "Error cargando el pedido".to_string())
        }
    }
}

async fn line_status(
    path: web::Path<(Uuid, Uuid)>,
    form: web::Form<LineStatusForm>,
    pool_data: web::Data<DbPool>,
) -> HttpResponse {
    let (order_id, item_id) = path.into_inner();
    let result = set_line_status(None, &order_id, &item_id, form.status, pool_data.get_ref()).await;
    back(&format!("{}/{}", ADMIN_ORDERS, order_id), result)
}
