use askama::Template;
use crate::models::{
    data_transfer_objects::product::{Product,RoutesProductPanelGroup},
    models_x::ProductForPage1
};
use serde_json::{Value, from_value};
use crate::controllers::components::cart_add_controller::ROUTES as CART_ROUTES;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use bigdecimal::BigDecimal;
use crate::models::components::stars::Stars;
use crate::models::components::price::Price;
use crate::models::components::quantity::Quantity;
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
    /// Código ISO ("EUR"), no el símbolo: lo pone `Price` al pintar.
    pub currency: String,
    pub store_name: String,
    pub store_id: Uuid,
    pub images_product: Vec<ImageProduct>,
    pub variant_map: Vec<VariantMapItem>,
    pub variations_with_stock_status: Vec<Variation>,
    pub rating_average: f64,
    pub rating_count: i64
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
    /// `None` = la variación no tiene fila en `prices`. Es un `Option` para que
    /// UNA variación sin precio no tumbe el mapa entero: sin él, `from_value`
    /// fallaba para toda la lista y la ficha se quedaba sin variaciones.
    pub price: Option<BigDecimal>,
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
            rating_average: 0.0,
            rating_count: 0,
        }
    }
}

impl ProductDetailPrincipalData {
    /// Las estrellas de la media, para la cabecera.
    pub fn stars(&self) -> Stars {
        Stars::new(self.rating_average)
    }

    /// "9 opiniones", "1 opinión".
    pub fn rating_count_label(&self) -> String {
        if self.rating_count == 1 {
            "1 opinión".to_string()
        } else {
            format!("{} opiniones", self.rating_count)
        }
    }

    /// Precio de la variación que sale seleccionada al entrar (la primera del
    /// mapa, que viene ordenado por precio con los NULL al final). `None` si no
    /// hay variaciones o ni esa tiene precio.
    pub fn initial_price(&self) -> Option<&BigDecimal> {
        self.variant_map.first().and_then(|v| v.price.as_ref())
    }

    /// El precio grande de la cabecera: el de la variación que sale
    /// seleccionada, o "No disponible". El JS lo cambia al elegir otra.
    pub fn price_tag(&self) -> Price {
        Price::maybe(self.initial_price(), &self.currency).lg()
    }

    /// Cantidad a añadir, de 1 a 99. El tope real es el stock de la variación
    /// elegida, y ese lo comprueba el servidor al añadir.
    pub fn quantity_tag(&self) -> Quantity {
        Quantity::new(1, 1, Some(99))
    }

    pub fn variant_map_json(&self) -> String {
        serde_json::to_string(&self.variant_map).unwrap_or_else(|_| "[]".to_string())
    }

    /// URL del endpoint de "añadir al carrito". No es un dato del producto,
    /// por eso es un método y no un campo: se lee de las rutas del carrito.
    pub fn cart_add_url(&self) -> &'static str {
        &CART_ROUTES.add
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

            images_product: parse_json(db.images_product, "images_product", &db.id)
                .unwrap_or(default.images_product),

            variant_map: parse_json(db.variant_map, "variant_map", &db.id)
                .unwrap_or(default.variant_map),

            variations_with_stock_status: parse_json(db.variations_with_stock_status, "variations_with_stock_status", &db.id)
                .unwrap_or(default.variations_with_stock_status),

            rating_average: db.rating_average,
            rating_count: db.rating_count,
        }
    }
}

/// Lee uno de los JSON que monta la consulta de la ficha. Si no encaja se usa
/// el valor por defecto, como antes, pero ahora queda en el log: con el `.ok()`
/// a secas, una variación sin precio dejaba la ficha sin variaciones y nadie se
/// enteraba de por qué.
fn parse_json<T: serde::de::DeserializeOwned>(value: Option<Value>, field: &str, product_id: &Uuid) -> Option<T> {
    match from_value::<T>(value?) {
        Ok(parsed) => Some(parsed),
        Err(e) => {
            println!("Ficha {}: no se pudo leer `{}`: {}", product_id, field, e);
            None
        }
    }
}