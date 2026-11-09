use askama::Template;
use crate::i18n::I18n;
use crate::models::components::nav1_model::*;
use crate::models::components::login_base_model::LoginProductData;

/// Página de "no encontrado" de la tienda (404): un título, una explicación y
/// el botón para volver a la portada. La usa la ficha de un producto que no
/// está a la venta; vale para cualquier otra página de la tienda.
#[derive(Template,Clone,Debug)]
#[template(path="pages/not_found/not_found.html")]
pub struct NotFoundPage{
    pub user_logged: bool,
    pub page_name:String,
    pub nav1:Nav1Data,
    pub login_base_data: LoginProductData,
    /// El idioma de la petición (src/i18n.rs): `<html lang>`, enlaces y textos.
    pub i18n: I18n,
    pub title:String,
    pub message:String
}
