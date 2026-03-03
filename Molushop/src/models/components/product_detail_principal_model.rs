use askama::Template;
use crate::models::{
    data_transfer_objects::product::{Product,RoutesProductPanelGroup},
    models_x::ProductForPage1
};
use serde_json::{Value, from_value};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use bigdecimal::BigDecimal;
#[derive(Template,Clone,Debug)]
#[template(path = "components/product_detail_principal/product_detail_principal.html")]
pub struct ProductDetailPrincipal{
    product_data : ProductDetailPrincipalData
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductDetailPrincipalData{
    pub id: Uuid,
    pub name: String,
    pub brand: String,
    pub description: String,
    pub currency: String,
    pub store_name: String,
    pub store_id: Uuid,
    pub images_product: Vec<ImageProduct>,
    pub variant_map: Vec<VariantMapItem>,
    pub variations_with_stock_status: Vec<Variation>
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageProduct{
    pub is_main : bool,
    pub image_url : String
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Variation{
    pub name : String,
    pub values : Vec<VariationValue>
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariationValue{
    pub value : String,
    pub in_stock : bool,
    pub is_default : bool
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariantAttribute {
    pub name: String,
    pub value: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariantMapItem {
    pub id: Uuid,
    pub price: BigDecimal,
    pub stock: i32,
    pub attributes: Vec<VariantAttribute>,
}

// Implementación de Default para ImageProduct
impl Default for ImageProduct {
    fn default() -> Self {
        Self {
            is_main: true,
            image_url: "https://via.placeholder.com/500".to_string(),
        }
    }
}

// Implementación de Default para Variation
impl Default for VariationValue {
    fn default() -> Self {
        Self {
            value: "Talla".to_string(),
            in_stock: false,
            is_default: false,
        }
    }
}

// Implementación de Default para Variation
impl Default for Variation {
    fn default() -> Self {
        Self {
            name: "Talla".to_string(),
            values : Vec::new()
        }
    }
}

// Implementación de Default para el modelo principal
impl Default for ProductDetailPrincipalData {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Producto genérico".to_string(),
            brand: "Marca blanca".to_string(),
            description: "Sin descripción disponible".to_string(),
            currency: "EUR".to_string(),
            store_name: "Tienda Principal".to_string(),
            store_id: Uuid::nil(),
            images_product: Vec::new(),
            variant_map: Vec::new(),
            variations_with_stock_status: Vec::new(),
        }
    }
}

impl ProductDetailPrincipalData {
    pub fn variant_map_json(&self) -> String {
        serde_json::to_string(&self.variant_map).unwrap_or_else(|_| "[]".to_string())
    }
}

impl From<ProductForPage1> for ProductDetailPrincipalData {
    fn from(db: ProductForPage1) -> Self {
        let default = ProductDetailPrincipalData::default();

        Self {
            id: db.id,
            name: db.name.unwrap_or(default.name),
            brand: db.brand.unwrap_or(default.brand),
            description: db.description.unwrap_or(default.description),
            currency: db.currency.unwrap_or(default.currency),
            store_name: db.store_name.unwrap_or(default.store_name),
            store_id: db.store_id.unwrap_or(default.store_id),

            images_product: db.images_product
                .and_then(|v| from_value::<Vec<ImageProduct>>(v).ok())
                .unwrap_or(default.images_product),

            variant_map: db.variant_map
                .and_then(|v| from_value::<Vec<VariantMapItem>>(v).ok())
                .unwrap_or(default.variant_map),

            variations_with_stock_status: db.variations_with_stock_status
                .and_then(|v| from_value::<Vec<Variation>>(v).ok())
                .unwrap_or(default.variations_with_stock_status),
        }
    }
}