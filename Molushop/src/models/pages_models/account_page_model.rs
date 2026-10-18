use askama::Template;
use crate::models::components::nav1_model::*;
use crate::models::components::login_base_model::LoginProductData;

/// "Mi cuenta" (`/account`): cambiar la contraseña y cerrar las demás sesiones.
/// Sin sesión solo enseña el botón de identificarse.
#[derive(Template,Clone,Debug)]
#[template(path="pages/account/account.html")]
pub struct AccountPage{
    pub user_logged: bool,
    pub page_name:String,
    pub nav1:Nav1Data,
    pub login_base_data:LoginProductData,
    pub username: String,
    /// Sesiones abiertas en otros sitios (no revocadas y sin caducar).
    pub other_sessions: i64,
    /// Aviso del último POST (viene de `?ok=` / `?error=`).
    pub ok_message: Option<String>,
    pub error_message: Option<&'static str>
}
