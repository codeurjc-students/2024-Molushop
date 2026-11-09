//vam
use askama::Template;
use crate::i18n::I18n;
use crate::models::components::nav1_model::*;
use crate::models::components::login_base_model::LoginProductData;
use crate::models::pages_models::search_page_model::CategoryLink;

#[derive(Template,Clone,Debug)]
#[template(path="pages/home/home.html")]
pub struct Home{
    pub user_logged: bool,
    pub page_name:String,
    pub group_cards2:String,
    /// Las categorías de arriba del todo con productos, cada una a su búsqueda.
    pub categories:Vec<CategoryLink>,
    pub nav1:Nav1Data,
    pub login_base_data: LoginProductData,
    /// El idioma de la petición (src/i18n.rs): `<html lang>`, enlaces y textos.
    pub i18n: I18n
}


//aqui los elementos de la pagina? tipo variables de componentes?
