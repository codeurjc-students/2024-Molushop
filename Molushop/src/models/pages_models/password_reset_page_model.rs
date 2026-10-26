use askama::Template;
use crate::i18n::I18n;
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
    pub login_base_data: LoginProductData,
    /// El idioma de la petición (src/i18n.rs): `<html lang>`, enlaces y textos.
    pub i18n: I18n,
    pub token: String,
    pub token_valid: bool,
    pub done: bool,
    pub error_message: Option<String>
}
