use askama::Template;
use crate::i18n::I18n;
use crate::models::components::nav1_model::*;
use crate::models::components::login_base_model::LoginProductData;

/// Directorio de tiendas (`/stores`).
#[derive(Template,Clone,Debug)]
#[template(path="pages/stores/stores.html")]
pub struct StoresPage{
    pub user_logged: bool,
    pub page_name:String,
    pub nav1:Nav1Data,
    pub login_base_data: LoginProductData,
    /// El idioma de la petición (src/i18n.rs): `<html lang>`, enlaces y textos.
    pub i18n: I18n,
    /// Las tiendas activas, por nombre.
    pub stores: Vec<StoreCard>,
    pub home_url: &'static str,
}

/// Una tarjeta del directorio.
#[derive(Clone,Debug)]
pub struct StoreCard{
    pub name: String,
    /// `/stores/{id}`, sin idioma: la plantilla le pone el prefijo con `i18n.url`.
    pub url: String,
    pub products: i64,
    /// Foto de su primer producto, o `NO_IMAGE`.
    pub image: String,
}
