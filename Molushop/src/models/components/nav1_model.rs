use askama::Template;

use serde::{Serialize,Deserialize};
use crate::models::components::product_card_model::ProductCardData;
use crate::models::pages_models::search_page_model::CategoryLink;


#[derive(Template,Clone,Debug)]
#[template(path="components/nav1/templates/nav1.html")]
pub struct Nav1<'a>{
    //pub login_base_data: Nav1Data
    pub nav1: Nav1Data,
    /// nav1.html enlaza en el idioma de la página (en las páginas lo pone la página).
    pub i18n: &'a crate::i18n::I18n,
}

#[derive(Debug, Clone)]
pub struct Nav1Data{
    pub routes: &'static Routes,
    pub name: String,
    pub cart_count: i64,
    /// Si hay sesión. El menú de perfil enseña "Identifícate" o las entradas de la cuenta
    /// según esto, en vez de deducirlo de que `name` venga vacío.
    pub logged: bool,
    /// Lo buscado, para dejarlo escrito en el buscador (solo en `/search`).
    pub search: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Routes{
    pub home: String,
    pub cart: String,
    pub orders: String,
    pub favorites: String,
    pub account: String,
    pub logout: String,
    pub search: String,
    /// Las sugerencias que salen bajo el buscador mientras se escribe.
    pub suggestions: String
}

/// Sugerencias del buscador (`/components/search-suggestions?q=`): categorías
/// cuyo nombre encaja, los primeros productos y el enlace a todos los resultados.
#[derive(Template,Clone,Debug)]
#[template(path="components/nav1/templates/suggestions.html")]
pub struct SearchSuggestions<'a>{
    pub query: String,
    pub categories: Vec<CategoryLink>,
    pub products: Vec<ProductCardData>,
    /// La búsqueda completa (`/search?q=`).
    pub all_url: String,
    /// El de la página desde la que se pide (`HX-Current-URL`).
    pub i18n: &'a crate::i18n::I18n,
}
