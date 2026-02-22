use askama::Template;

use serde::{Serialize,Deserialize};
use uuid::Uuid;
use bigdecimal::BigDecimal;
use crate::models::models_x::ProductCard1;
use crate::constants::urls::PRODUCT_URL_PREFIX;

#[derive(Template,Clone,Debug)]
#[template(path="components/product_card/templates/product_card.html")]
pub struct ProductCard{
    pub product: ProductCardData
}

#[derive(Debug, Clone)]
pub struct ProductCardData{
    pub name: String,
    pub image: String,
    pub price: BigDecimal,
    pub url : String
}

//hacer una funcion default de error
impl Default for ProductCardData {
    fn default() -> Self {
        Self {
            name: "Producto no disponible".to_string(),
            image: "/static/images/no-image.png".to_string(),
            price: BigDecimal::from(0),
            url : "javascript:void(0)".to_string()
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
            image: db.image_url.unwrap_or_else(|| "/images/default-placeholder.png".to_string()),
            
            // Si el precio es None, inicializamos un BigDecimal de 0
            price: db.price.unwrap_or_else(|| BigDecimal::from(0)),

            // Convertimos el UUID a una URL con formato producto/uuid
            url: format!("{}{}",PRODUCT_URL_PREFIX,db.id)
        }
    }
}
//lamar a una constante 
