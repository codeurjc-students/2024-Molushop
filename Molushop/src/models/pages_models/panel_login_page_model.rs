use askama::Template;
use crate::models::components::panel_login_v1::PanelLoginData;

/// `/panel/login`. Sin nav1 ni modal de login: el panel no es la tienda.
#[derive(Template,Clone,Debug)]
#[template(path="pages/panel_login/panel_login.html")]
pub struct PanelLoginPage{
    pub user_logged: bool,
    pub page_name:String,
    pub panel_login: PanelLoginData
}
