use askama::Template;
use crate::models::components::panel_nav::PanelNav;
use crate::services::panel_orders_service::PanelOrder;

/// Listado de pedidos del panel. Lo usa la tienda (`/panel/stores/{id}/orders`,
/// solo con sus líneas).
#[derive(Template,Clone,Debug)]
#[template(path="pages/panel/orders.html")]
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

/// Ficha de un pedido en el panel, con el cambio de estado de cada línea.
#[derive(Template,Clone,Debug)]
#[template(path="pages/panel/order.html")]
pub struct PanelOrderPage{
    pub user_logged: bool,
    pub page_name:String,
    pub panel_nav: PanelNav,
    /// Listado al que vuelve y prefijo del POST de cada línea.
    pub base_url: String,
    pub all_lines: bool,
    pub order: PanelOrder,
    /// Aviso del último POST (viene de `?ok=` / `?error=`).
    pub ok_message: Option<&'static str>,
    pub error_message: Option<&'static str>
}
