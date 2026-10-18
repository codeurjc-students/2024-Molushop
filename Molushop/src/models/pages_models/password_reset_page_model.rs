use askama::Template;
use crate::models::components::nav1_model::*;
use crate::models::components::login_base_model::LoginProductData;

/// `/password/reset?token=...`: el enlace del email. Formulario de la nueva
/// contraseña si el token vale; si no, "enlace caducado". Tras cambiarla
/// (`done`), el aviso y el botón de identificarse.
#[derive(Template,Clone,Debug)]
#[template(path="pages/password_reset/password_reset.html")]
pub struct PasswordResetPage{
    pub user_logged: bool,
    pub page_name:String,
    pub nav1:Nav1Data,
    pub login_base_data:LoginProductData,
    pub token: String,
    pub token_valid: bool,
    pub done: bool,
    pub error_message: Option<&'static str>
}
