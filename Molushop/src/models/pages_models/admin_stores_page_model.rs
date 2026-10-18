use askama::Template;
use crate::models::components::panel_nav::PanelNav;
use crate::services::admin_stores_service::AdminStore;

/// `/admin/stores`: todas las tiendas con su estado, sus miembros y las
/// acciones del admin (aprobar, rechazar, desactivar, crear, asignar usuarios).
#[derive(Template,Clone,Debug)]
#[template(path="pages/admin_stores/admin_stores.html")]
pub struct AdminStoresPage{
    pub user_logged: bool,
    pub page_name:String,
    pub panel_nav: PanelNav,
    pub stores: Vec<AdminStore>,
    /// Aviso del último POST (viene de `?ok=` / `?error=`).
    pub ok_message: Option<&'static str>,
    pub error_message: Option<&'static str>
}
