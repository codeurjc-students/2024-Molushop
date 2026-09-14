use askama::Template;
use bigdecimal::BigDecimal;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::constants::money::{DEFAULT_CURRENCY, symbol_of};
use crate::constants::urls::CART_URL;
use crate::models::models_x::{Order, OrderItem};

/// Etiqueta de `orders.status`. Vive suelta y no como método porque la usan la
/// ficha del pedido y la lista de "mis pedidos": si cada una tuviera su `match`,
/// añadir un estado nuevo arreglaría una y dejaría la otra mintiendo en silencio.
pub fn status_label(status: i16) -> &'static str {
    match status {
        1 => "Confirmado",
        2 => "Cancelado",
        _ => "Pendiente"
    }
}

#[derive(Template,Clone,Debug)]
#[template(path = "components/order_detail_v1/order_detail_v1.html")]
pub struct OrderDetailV1{
    pub order: OrderData
}

/// Una línea del pedido tal como se pinta. Sale del snapshot de `order_items`,
/// NO del catálogo: si el producto cambia de nombre o de precio mañana, el
/// pedido sigue diciendo lo que se compró.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderLineData{
    pub product_name: String,
    pub variation_label: String,
    pub sku: String,
    pub image: String,
    pub store_name: String,
    pub unit_price: BigDecimal,
    pub quantity: i32,
    pub line_total: BigDecimal
}

#[derive(Serialize, Debug, Clone)]
pub struct OrderData{
    /// False cuando el pedido no existe o no es de quien lo pide. La plantilla
    /// enseña entonces un aviso en vez de la ficha.
    pub found: bool,
    /// Si quien pide la ficha tiene sesión. Con `found = false` separa "identifícate"
    /// (sin sesión: se abre el login) de "ese pedido no existe o no es tuyo".
    pub logged: bool,
    /// True solo al llegar desde la confirmación del checkout (`?nuevo=1`): es lo único
    /// que decide si la cabecera dice "¡Pedido confirmado!". Lo rellena la ruta.
    pub is_new: bool,
    pub id: Uuid,
    pub order_number: i64,
    pub status: i16,
    pub currency: String,
    pub subtotal: BigDecimal,
    pub discount_total: BigDecimal,
    pub shipping_total: BigDecimal,
    pub total_amount: BigDecimal,
    pub ship_street1: String,
    pub ship_street2: String,
    pub ship_postal_code: String,
    pub ship_city: String,
    pub ship_province: String,
    pub items: Vec<OrderLineData>,
    pub cart_url: &'static str
}

impl Default for OrderData {
    fn default() -> Self {
        Self {
            found: false,
            logged: false,
            is_new: false,
            id: Uuid::nil(),
            order_number: 0,
            status: 0,
            currency: DEFAULT_CURRENCY.to_string(),
            subtotal: BigDecimal::from(0),
            discount_total: BigDecimal::from(0),
            shipping_total: BigDecimal::from(0),
            total_amount: BigDecimal::from(0),
            ship_street1: String::new(),
            ship_street2: String::new(),
            ship_postal_code: String::new(),
            ship_city: String::new(),
            ship_province: String::new(),
            items: Vec::new(),
            cart_url: CART_URL
        }
    }
}

impl OrderData {
    /// Con sesión, pero el pedido no existe o no es suyo.
    pub fn not_found_logged() -> Self {
        Self { logged: true, ..Default::default() }
    }

    /// Símbolo para pintar. El campo guarda el código ISO, igual que en el
    /// carrito y que en la BD.
    pub fn currency_symbol(&self) -> String {
        symbol_of(&self.currency)
    }

    pub fn status_label(&self) -> &'static str {
        status_label(self.status)
    }

    /// Nº de unidades, no de líneas.
    pub fn total_units(&self) -> i32 {
        self.items.iter().map(|it| it.quantity).sum()
    }

    pub fn from_db(order: Order, items: Vec<OrderItem>) -> Self {
        let default = OrderData::default();

        Self {
            found: true,
            logged: true,
            is_new: false,
            id: order.id,
            order_number: order.order_number,
            status: order.status,
            currency: order.currency.trim().to_string(),
            subtotal: order.subtotal,
            discount_total: order.discount_total,
            shipping_total: order.shipping_total,
            total_amount: order.total_amount,
            ship_street1: order.ship_street1,
            ship_street2: order.ship_street2.unwrap_or_default(),
            ship_postal_code: order.ship_postal_code,
            ship_city: order.ship_city,
            ship_province: order.ship_province,
            items: items.into_iter().map(OrderLineData::from).collect(),
            cart_url: default.cart_url
        }
    }
}

impl From<OrderItem> for OrderLineData {
    fn from(db: OrderItem) -> Self {
        Self {
            product_name: db.product_name,
            variation_label: db.variation_label.unwrap_or_default(),
            sku: db.sku.unwrap_or_default(),
            image: db.image_url.unwrap_or_else(|| "/images/default-placeholder.png".to_string()),
            store_name: db.store_name.unwrap_or_default(),
            unit_price: db.unit_price,
            quantity: db.quantity,
            line_total: db.line_total
        }
    }
}
