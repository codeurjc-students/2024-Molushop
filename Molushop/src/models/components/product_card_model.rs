use askama::Template;

use uuid::Uuid;
use bigdecimal::BigDecimal;
use crate::models::models_x::ProductCard1;
use crate::constants::urls::PRODUCT_URL_PREFIX;
use crate::models::components::stars::Stars;
use crate::models::components::price::Price;
use crate::constants::money::DEFAULT_CURRENCY;
use crate::i18n::I18n;
use crate::models::components::favorite_button::FavoriteButton;

/// La imagen de los productos que no tienen ninguna.
pub const NO_IMAGE: &str = "/assets/imgs/no-image.svg";

#[derive(Template,Clone,Debug)]
#[template(path="components/product_card/templates/product_card.html")]
pub struct ProductCard<'a>{
    pub product: ProductCardData,
    pub i18n: &'a I18n,
}

#[derive(Debug, Clone)]
pub struct ProductCardData{
    pub id: Uuid,
    pub name: String,
    pub image: String,
    /// `None` = ninguna variación tiene precio: se pinta "No disponible".
    pub price: Option<BigDecimal>,
    /// Código ISO ("EUR"); el símbolo lo pone Price.
    pub currency: String,
    pub url : String,
    pub rating_average: f64,
    pub rating_count: i64,
    /// Si el usuario lo tiene en favoritos. No sale de la consulta de las
    /// tarjetas: lo pone `mark_favorites` (product_card_group_service).
    pub is_favorite: bool,
    /// `false` = ya no se vende (borrado, inactivo o de una tienda inactiva).
    /// Solo sale así en "Mis favoritos": la tarjeta no enlaza a la ficha, que
    /// daría 404, y deja el corazón para quitarlo.
    pub available: bool,
}

impl ProductCardData {
    /// El corazón de favoritos, en la esquina de la imagen.
    pub fn favorite_button(&self) -> FavoriteButton {
        FavoriteButton::new(self.id, self.is_favorite)
    }

    /// Las estrellas de la media, en pequeño y en el idioma de la página.
    pub fn stars(&self, i18n: &I18n) -> Stars {
        Stars::sm(self.rating_average).in_lang(i18n.lang)
    }

    pub fn price_tag(&self) -> Price {
        Price::maybe(self.price.as_ref(), &self.currency)
    }
}

//hacer una funcion default de error
impl Default for ProductCardData {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            name: "Producto no disponible".to_string(),
            image: NO_IMAGE.to_string(),
            price: None,
            currency: DEFAULT_CURRENCY.to_string(),
            url : "javascript:void(0)".to_string(),
            rating_average: 0.0,
            rating_count: 0,
            is_favorite: false,
            available: true,
        }
    }
}
//se podrá usar una constane para hacer la concatenación y asi formar la URL del producto

impl From<ProductCard1> for ProductCardData {
    fn from(db: ProductCard1) -> Self {
        Self {
            id: db.id,
            // Si el nombre es None, usamos un string vacío o un valor por defecto
            name: db.name.unwrap_or_else(|| "Producto sin nombre".to_string()),
            
            // Mapeamos image_url a image
            image: db.image_url.unwrap_or_else(|| NO_IMAGE.to_string()),
            
            // Sin precio se pinta "No disponible", no "0 €".
            price: db.price,
            currency: db.currency.unwrap_or_else(|| DEFAULT_CURRENCY.to_string()),

            // Convertimos el UUID a una URL con formato producto/uuid
            url: format!("{}{}",PRODUCT_URL_PREFIX,db.id),

            rating_average: db.rating_average,
            rating_count: db.rating_count,
            is_favorite: false,
            available: true,
        }
    }
}
//lamar a una constante 

#[cfg(test)]
mod tests {
    use super::*;

    fn render(is_favorite: bool) -> String {
        let product = ProductCardData { id: Uuid::new_v4(), is_favorite, ..Default::default() };
        ProductCard { product, i18n: I18n::default_ref() }.render().unwrap()
    }

    /// Dentro del enlace, pulsar el corazón llevaría también a la ficha.
    #[test]
    fn heart_is_outside_the_link() {
        let html = render(false);
        let heart = html.find("fav-btn").unwrap();
        let link = html.find("class=\"link\"").unwrap();
        assert!(heart < link, "el corazón tiene que ir antes (y fuera) del enlace a la ficha");
    }

    #[test]
    fn heart_is_filled_or_empty() {
        let empty = render(false);
        assert!(empty.contains("/components/favorites/add/"));
        assert!(empty.contains("aria-pressed=\"false\""));
        let full = render(true);
        assert!(full.contains("fav-btn is-active"));
        assert!(full.contains("/components/favorites/remove/"));
        assert!(full.contains("icon-heart-filled"));
    }

    /// La que ya no se vende no enlaza (la ficha daría 404), pero deja quitarla de favoritos.
    #[test]
    fn unavailable_card_has_no_link_and_can_be_removed() {
        let product = ProductCardData { id: Uuid::new_v4(), is_favorite: true, available: false, ..Default::default() };
        let html = ProductCard { product, i18n: I18n::default_ref() }.render().unwrap();
        assert!(html.contains("product-card is-unavailable"));
        assert!(!html.contains("<a class=\"link\""), "sin enlace: la ficha daría 404");
        assert!(html.contains("Ya no está a la venta"));
        assert!(html.contains("/components/favorites/remove/"));
    }
}
