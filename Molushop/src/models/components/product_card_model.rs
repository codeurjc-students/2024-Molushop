use askama::Template;

use serde::{Serialize,Deserialize};
use uuid::Uuid;
use bigdecimal::BigDecimal;
use crate::models::models_x::ProductCard1;
use crate::constants::urls::PRODUCT_URL_PREFIX;
use crate::models::components::stars::Stars;
use crate::models::components::price::Price;
use crate::constants::money::DEFAULT_CURRENCY;
use crate::i18n::I18n;

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
    pub name: String,
    pub image: String,
    /// `None` = ninguna variación tiene precio: se pinta "No disponible".
    pub price: Option<BigDecimal>,
    /// Código ISO ("EUR"); el símbolo lo pone Price.
    pub currency: String,
    pub url : String,
    pub rating_average: f64,
    pub rating_count: i64
}

impl ProductCardData {
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
            name: "Producto no disponible".to_string(),
            image: NO_IMAGE.to_string(),
            price: None,
            currency: DEFAULT_CURRENCY.to_string(),
            url : "javascript:void(0)".to_string(),
            rating_average: 0.0,
            rating_count: 0
        }
    }
}
//se podrá usar una constane para hacer la concatenación y asi formar la URL del producto

impl From<ProductCard1> for ProductCardData {
    fn from(db: ProductCard1) -> Self {
        Self {
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
            rating_count: db.rating_count
        }
    }
}
//lamar a una constante 
