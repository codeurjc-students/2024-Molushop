use askama::Template;
use crate::models::components::panel_nav::PanelNav;

/// `/panel`: portada con accesos a las secciones.
#[derive(Template,Clone,Debug)]
#[template(path="pages/panel/home.html")]
pub struct PanelHomePage{
    pub user_logged: bool,
    pub page_name:String,
    pub panel_nav: PanelNav
}

/// `/admin`: portada de lo exclusivo del admin (aún sin secciones).
#[derive(Template,Clone,Debug)]
#[template(path="pages/panel/admin.html")]
pub struct AdminHomePage{
    pub user_logged: bool,
    pub page_name:String,
    pub panel_nav: PanelNav
}
