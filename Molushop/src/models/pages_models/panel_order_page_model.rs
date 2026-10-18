use askama::Template;
use crate::models::components::panel_nav::PanelNav;
use crate::services::panel_orders_service::PanelOrder;

/// Ficha de un pedido en el panel, con el cambio de estado de cada línea.
#[derive(Template,Clone,Debug)]
#[template(path="pages/panel_order/panel_order.html")]
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
