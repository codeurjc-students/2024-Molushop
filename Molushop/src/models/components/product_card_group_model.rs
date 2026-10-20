use askama::Template;
use crate::models::components::product_card_model::*;

#[derive(Template,Clone,Debug)]
#[template(path="components/product_card_group/templates/product_card_group.html")]
pub struct ProductCardGroup{
    pub products: Vec<ProductCardData>,
    /// `Some`: hay más productos y el botón "Cargar más" los pide aquí.
    pub more_url: Option<String>,
}

/// Una tanda de "Cargar más": las tarjetas y, si quedan, el botón siguiente,
/// sin la caja del grupo (van dentro de la que ya está en la página).
#[derive(Template,Clone,Debug)]
#[template(path="components/product_card_group/templates/items.html")]
pub struct ProductCardGroupPage{
    pub products: Vec<ProductCardData>,
    pub more_url: Option<String>,
}
