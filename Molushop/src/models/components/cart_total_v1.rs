use askama::Template;
use bigdecimal::BigDecimal;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::models_x::CartItem1;
use crate::constants::urls::PRODUCT_URL_PREFIX;
use crate::constants::money::DEFAULT_CURRENCY;
use crate::models::components::price::Price;
use crate::models::components::quantity::Quantity;
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
    /// Código ISO tal cual está en `prices.currency` ("EUR"), NO el símbolo.
    /// Para pintarlo usa `total_tag()` y compañía (átomo `Price`).
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
    /// Código ISO ("EUR"), no el símbolo. Ver `price_tag()`.
    pub currency: String,
    pub store_name: String,
    pub quantity: i32,
    pub stock: i32,
    /// False si ya no se puede comprar: sin tienda activa, producto eliminado o
    /// inactivo, variación desactivada o sin precio. Son los mismos casos en que el checkout
    /// rechaza el pedido; el carrito la marca, no la cuenta y no deja elegirla.
    pub available: bool
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
            currency: DEFAULT_CURRENCY.to_string()
        }
    }
}

impl CartTotalData {
    /// Los totales solo cuentan las líneas disponibles.
    pub fn from_items(items: Vec<CartItemData>) -> Self {
        let default = CartTotalData::default();

        let total: BigDecimal = items
            .iter()
            .filter(|it| it.available)
            .fold(BigDecimal::from(0), |acc, it| acc + it.subtotal.clone());
        let total_units: i32 = items.iter().filter(|it| it.available).map(|it| it.quantity).sum();
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

    // Los importes del resumen, con el átomo de precio. La moneda se guarda
    // como código ISO y el símbolo lo pone Price al pintar.
    pub fn total_tag(&self) -> Price {
        Price::new(&self.total, &self.currency)
    }

    pub fn discount_tag(&self) -> Price {
        Price::new(&self.discount, &self.currency)
    }

    pub fn final_total_tag(&self) -> Price {
        Price::new(&self.final_total, &self.currency)
    }
}

impl CartItemData {
    /// Precio unitario de la línea (el carrito).
    pub fn price_tag(&self) -> Price {
        Price::new(&self.price, &self.currency)
    }

    /// Precio x cantidad (el checkout).
    pub fn subtotal_tag(&self) -> Price {
        Price::new(&self.subtotal, &self.currency)
    }

    /// El selector de la línea, de 0 al stock. Si el stock ya es menor que
    /// lo que hay en el carrito, el tope es la cantidad actual: así el input
    /// enseña lo que de verdad hay (y lo que suma el total), no el stock.
    pub fn quantity_tag(&self) -> Quantity {
        Quantity::new(self.quantity, 0, Some(self.stock.max(self.quantity))).sm()
    }
}

/// Normaliza lo que venga de la BD a un código ISO. Un precio sin moneda cae al
/// de por defecto; la conversión a símbolo NO se hace aquí (ver `currency_symbol`).
fn currency_code(code: Option<String>) -> String {
    match code.as_deref().map(|c| c.trim()) {
        Some(c) if !c.is_empty() => c.to_uppercase(),
        _ => DEFAULT_CURRENCY.to_string()
    }
}

impl From<CartItem1> for CartItemData {
    fn from(db: CartItem1) -> Self {
        let available = db.on_sale && db.store_name.is_some() && db.price.is_some();
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
            currency: currency_code(db.currency),
            store_name: db.store_name.unwrap_or_default(),
            quantity: db.quantity,
            stock: db.stock,
            available
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn line(price: &str, quantity: i32, available: bool) -> CartItemData {
        let price = BigDecimal::from_str(price).unwrap();
        CartItemData {
            id: Uuid::new_v4(),
            product_var_id: Uuid::new_v4(),
            name: String::new(),
            variation_label: String::new(),
            image: String::new(),
            url: String::new(),
            subtotal: price.clone() * BigDecimal::from(quantity),
            price,
            currency: "EUR".to_string(),
            store_name: String::new(),
            quantity,
            stock: 10,
            available,
        }
    }

    #[test]
    fn totals_skip_unavailable_lines() {
        let data = CartTotalData::from_items(vec![line("9.50", 2, true), line("100", 3, false)]);
        assert_eq!(data.total, BigDecimal::from_str("19.00").unwrap());
        assert_eq!(data.total_units, 2);
        assert_eq!(data.items.len(), 2, "la línea sigue en la lista, solo no cuenta");
    }
}
