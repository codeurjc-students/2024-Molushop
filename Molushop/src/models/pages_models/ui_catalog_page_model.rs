use askama::Template;
// La plantilla llama a Stars::new(...) para enseñar el átomo de estrellas.
use crate::models::components::stars::Stars;

/// Catálogo del sistema de diseño (`/ui`): las variables de `tokens.css` y los
/// átomos. Es una página estática; los dos campos son los que pide `layout1`.
#[derive(Template,Clone,Debug)]
#[template(path="pages/ui_catalog/ui_catalog.html")]
pub struct UiCatalogPage{
    pub user_logged: bool,
    pub page_name:String
}
