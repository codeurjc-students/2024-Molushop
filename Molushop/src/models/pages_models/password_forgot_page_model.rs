use askama::Template;
use crate::models::components::nav1_model::*;
use crate::models::components::login_base_model::LoginProductData;

/// `/password/forgot`: pedir el enlace para cambiar la contraseña.
#[derive(Template,Clone,Debug)]
#[template(path="pages/password_forgot/password_forgot.html")]
pub struct PasswordForgotPage{
    pub user_logged: bool,
    pub page_name:String,
    pub nav1:Nav1Data,
    pub login_base_data:LoginProductData,
    /// Tras el POST: el mismo aviso exista o no la cuenta.
    pub sent: bool,
    /// Lo que dura el enlace (`RESET_MINUTES`), para el aviso.
    pub reset_minutes: i64,
    pub error_message: Option<&'static str>
}
