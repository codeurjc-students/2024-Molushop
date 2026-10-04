use askama::Template;
use crate::models::components::panel_nav::PanelNav;
use crate::services::panel_role_service::PanelStore;

/// `/panel`: las tiendas del usuario, cada una con su estado.
#[derive(Template,Clone,Debug)]
#[template(path="pages/panel/home.html")]
pub struct PanelHomePage{
    pub user_logged: bool,
    pub page_name:String,
    pub panel_nav: PanelNav,
    pub stores: Vec<PanelStore>
}

/// `/admin`: lo exclusivo del admin. De momento, la lista de todas las tiendas.
#[derive(Template,Clone,Debug)]
#[template(path="pages/panel/admin.html")]
pub struct AdminHomePage{
    pub user_logged: bool,
    pub page_name:String,
    pub panel_nav: PanelNav,
    pub stores: Vec<PanelStore>
}
