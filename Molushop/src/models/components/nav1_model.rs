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

impl Nav1Data {
    pub fn cart_badge(&self) -> CartBadge {
        CartBadge::new(self.cart_count)
    }
}

/// El número del carrito del menú (`#cart-badge`). Con 0 no se ve. Las
/// respuestas que cambian el carrito lo mandan con `oob()` y htmx sustituye
/// el del menú; `bumped()` le da el bote (al añadir desde la ficha).
#[derive(Template, Clone, Debug)]
#[template(path = "components/nav1/templates/cart_badge.html")]
pub struct CartBadge {
    pub count: i64,
    pub bump: bool,
    pub oob: bool,
}

impl CartBadge {
    pub fn new(count: i64) -> Self {
        Self { count, bump: false, oob: false }
    }

    pub fn bumped(mut self) -> Self {
        self.bump = true;
        self
    }

    /// Para ir detrás de otra respuesta (`hx-swap-oob`).
    pub fn oob(mut self) -> String {
        self.oob = true;
        self.render().unwrap_or_default()
    }
}

impl askama::filters::HtmlSafe for CartBadge {}

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Con 0 se esconde; el de la respuesta lleva hx-swap-oob y, al añadir, el bote
    #[test]
    fn cart_badge_renders_hidden_at_zero_and_oob_with_bump() {
        let html = CartBadge::new(0).render().unwrap();
        assert!(html.contains("style=\"display:none;\""), "{}", html);
        assert!(!html.contains("hx-swap-oob"), "{}", html);

        let html = CartBadge::new(3).bumped().oob();
        assert!(html.contains("id=\"cart-badge\""), "{}", html);
        assert!(html.contains("cart-badge is-bumped"), "{}", html);
        assert!(html.contains("hx-swap-oob=\"true\""), "{}", html);
        assert!(html.contains(">3</span>"), "{}", html);
        assert!(!html.contains("display:none"), "{}", html);
    }
}
