use askama::Template;
use crate::models::components::panel_nav::PanelNav;
use crate::services::panel_orders_service::PanelOrder;

/// Listado de pedidos del panel. Lo usan la tienda (`/panel/stores/{id}/orders`,
/// solo con sus líneas) y el admin (`/admin/orders`, con todas).
#[derive(Template,Clone,Debug)]
#[template(path="pages/panel_orders/panel_orders.html")]
pub struct PanelOrdersPage{
    pub user_logged: bool,
    pub page_name:String,
    pub panel_nav: PanelNav,
    pub title: String,
    /// Prefijo de la ficha: `{base_url}/{order_id}`.
    pub base_url: String,
    /// False cuando solo se ven las líneas de una tienda: la página lo avisa.
    pub all_lines: bool,
    pub orders: Vec<PanelOrder>
}
