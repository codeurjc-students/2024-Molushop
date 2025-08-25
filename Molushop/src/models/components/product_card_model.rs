use rinja::Template;

use serde::{Serialize,Deserialize};
use uuid::Uuid;
use bigdecimal::BigDecimal;

#[derive(Template,Clone,Debug)]
#[template(path="components/product_card/templates/product_card.html")]
pub struct ProductCard{
    pub product: ProductCardData
}

#[derive(Debug, Clone)]
pub struct ProductCardData{
    pub name: String,
    pub image: String,
    pub price: BigDecimal
}

//hacer una funcion default de error
impl Default for ProductCardData {
    fn default() -> Self {
        Self {
            name: "Producto no disponible".to_string(),
            image: "/static/images/no-image.png".to_string(),
            price: BigDecimal::from(0)
        }
    }
}