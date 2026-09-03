use askama::Template;
use bigdecimal::BigDecimal;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::models_x::CartItem1;
use crate::constants::urls::PRODUCT_URL_PREFIX;
use crate::controllers::components::cart_add_controller::ROUTES;

#[derive(Template,Clone,Debug)]
#[template(path = "components/cart_total_v1/cart_total_v1.html")]
pub struct CartTotalV1{
    pub cart_total: CartTotalData
}

/// Sólo el cuerpo del componente, sin los <link>/<script> del wrapper.
/// Es lo que se devuelve en los refrescos parciales: si se reinyectase el
/// wrapper, el script.js se volvería a cargar en cada actualización.
#[derive(Template,Clone,Debug)]
#[template(path = "components/cart_total_v1/templates/base.html")]
pub struct CartTotalV1Body{
    pub cart_total: CartTotalData
}

#[derive(Serialize, Debug, Clone)]
pub struct CartRoutes{
    pub add: String,
    pub update: String,
    pub remove: String,
    /// Página de checkout. No es un endpoint del componente, pero se expone aquí
    /// porque el botón "Continuar" del resumen es quien lleva a ella.
    pub checkout: String
}

#[derive(Serialize, Debug, Clone)]
pub struct CartTotalData{
    pub routes: &'static CartRoutes,
    pub items: Vec<CartItemData>,
    /// Suma de los subtotales de todas las líneas
    pub total: BigDecimal,
    /// Descuento total aplicado. De momento siempre 0: no hay lógica de descuentos de carrito.
    pub discount: BigDecimal,
    /// total - discount
    pub final_total: BigDecimal,
    /// Nº de unidades (no de líneas)
    pub total_units: i32,
    pub currency: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CartItemData{
    /// id de la fila en cart_products, es el identificador de la línea
    pub id: Uuid,
    pub product_var_id: Uuid,
    pub name: String,
    pub variation_label: String,
    pub image: String,
    pub url: String,
    pub price: BigDecimal,
    pub subtotal: BigDecimal,
    pub currency: String,
    pub store_name: String,
    pub quantity: i32,
    pub stock: i32
}

impl Default for CartTotalData {
    fn default() -> Self {
        Self {
            routes: &ROUTES,
            items: Vec::new(),
            total: BigDecimal::from(0),
            discount: BigDecimal::from(0),
            final_total: BigDecimal::from(0),
            total_units: 0,
            currency: "€".to_string()
        }
    }
}

impl CartTotalData {
    pub fn from_items(items: Vec<CartItemData>) -> Self {
        let default = CartTotalData::default();

        let total: BigDecimal = items
            .iter()
            .fold(BigDecimal::from(0), |acc, it| acc + it.subtotal.clone());
        let total_units: i32 = items.iter().map(|it| it.quantity).sum();
        let currency = items
            .first()
            .map(|it| it.currency.clone())
            .unwrap_or(default.currency);

        let discount = BigDecimal::from(0);
        let final_total = total.clone() - discount.clone();

        Self { routes: &ROUTES, items, total, discount, final_total, total_units, currency }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Traduce el código ISO de la moneda a su símbolo. Si no lo conoce, devuelve el código.
fn currency_symbol(code: Option<String>) -> String {
    match code.as_deref().map(|c| c.trim()) {
        Some("EUR") => "€".to_string(),
        Some("USD") => "$".to_string(),
        Some("GBP") => "£".to_string(),
        Some(other) if !other.is_empty() => other.to_string(),
        _ => "€".to_string()
    }
}

impl From<CartItem1> for CartItemData {
    fn from(db: CartItem1) -> Self {
        let price = db.price.unwrap_or_else(|| BigDecimal::from(0));
        // Si la consulta no pudo calcular el subtotal (producto sin precio) lo recalculamos aquí
        let subtotal = db.subtotal
            .unwrap_or_else(|| price.clone() * BigDecimal::from(db.quantity));

        Self {
            id: db.id,
            product_var_id: db.product_var_id,
            name: db.name.unwrap_or_else(|| "Producto sin nombre".to_string()),
            variation_label: db.variation_label.unwrap_or_default(),
            image: db.image_url.unwrap_or_else(|| "/images/default-placeholder.png".to_string()),
            url: format!("{}{}", PRODUCT_URL_PREFIX, db.product_id),
            price,
            subtotal,
            currency: currency_symbol(db.currency),
            store_name: db.store_name.unwrap_or_default(),
            quantity: db.quantity,
            stock: db.stock
        }
    }
}
