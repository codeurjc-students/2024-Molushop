use askama::Template;

/// `/panel/register`: solicitar una tienda. Sin nav del panel: quien llega aquí
/// aún no tiene acceso.
#[derive(Template,Clone,Debug)]
#[template(path="pages/panel/register.html")]
pub struct PanelRegisterPage{
    pub user_logged: bool,
    pub page_name:String,
    /// `Some` si hay sesión: solo se pide el nombre de la tienda.
    pub logged_username: Option<String>,
    pub error: Option<&'static str>,
    // Lo que se había escrito, para no perderlo si hay un error.
    pub store_name: String,
    pub username: String,
    pub email: String
}
