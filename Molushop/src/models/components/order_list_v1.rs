use askama::Template;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::constants::money::symbol_of;
use crate::constants::urls::{HOME_URL, ORDER_URL_PREFIX};
use crate::models::models_x::{Order, OrderItem};
use crate::models::components::order_detail_v1::{OrderLineData, status_label};

#[derive(Template,Clone,Debug)]
#[template(path = "components/order_list_v1/order_list_v1.html")]
pub struct OrderListV1{
    pub order_list: OrderListData
}

/// Un pedido tal como se pinta EN LA LISTA. No se reutiliza `OrderData` a
/// propósito: la lista no enseña la dirección ni el desglose del resumen, y
/// arrastrar esos campos daría a entender que la lista los pinta.
///
/// Las líneas sí son las mismas `OrderLineData` de la ficha: salen del mismo
/// snapshot de `order_items` y ya tienen su `From<OrderItem>`.
// Sin `Serialize`, al contrario que `OrderData`: `created_at` es un
// `NaiveDateTime` y chrono no trae la feature `serde` activada en este
// proyecto. La lista se pinta con Askama y nunca sale como JSON, así que no
// compensa tocar el Cargo.toml por un derive que nadie usa.
#[derive(Debug, Clone)]
pub struct OrderSummaryData{
    pub id: Uuid,
    pub order_number: i64,
    pub status: i16,
    pub currency: String,
    pub total_amount: BigDecimal,
    pub created_at: NaiveDateTime,
    pub items: Vec<OrderLineData>
}

impl OrderSummaryData {
    /// Símbolo para pintar; el campo guarda el código ISO, igual que en la ficha.
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

    /// Fecha corta. Es la primera del proyecto que se pinta: hasta ahora
    /// `created_at` se guardaba y no se enseñaba en ningún sitio.
    pub fn date_label(&self) -> String {
        self.created_at.format("%d/%m/%Y").to_string()
    }

    /// La ficha del pedido. Se compone aquí y no en la plantilla para que el
    /// prefijo siga saliendo de `constants::urls`, como en el resto del sitio.
    pub fn detail_url(&self) -> String {
        format!("{}{}", ORDER_URL_PREFIX, self.id)
    }

    pub fn from_db(order: Order, items: Vec<OrderItem>) -> Self {
        Self {
            id: order.id,
            order_number: order.order_number,
            status: order.status,
            currency: order.currency.trim().to_string(),
            total_amount: order.total_amount,
            created_at: order.created_at,
            items: items.into_iter().map(OrderLineData::from).collect()
        }
    }
}

/// La lista entera. Un visitante sin sesión y un usuario sin pedidos llegan los
/// dos con `orders` vacío; `logged` los separa: al primero se le pide que se
/// identifique (y se abre el login), al segundo se le dice que no ha comprado nada.
#[derive(Debug, Clone)]
pub struct OrderListData{
    pub orders: Vec<OrderSummaryData>,
    pub logged: bool,
    pub home_url: &'static str
}

impl Default for OrderListData {
    fn default() -> Self {
        Self {
            orders: Vec::new(),
            logged: false,
            home_url: HOME_URL
        }
    }
}

impl OrderListData {
    /// Con sesión y sin pedidos: "todavía no has hecho ningún pedido".
    pub fn empty_logged() -> Self {
        Self { logged: true, ..Default::default() }
    }

    pub fn from_db(rows: Vec<(Order, Vec<OrderItem>)>) -> Self {
        Self {
            orders: rows.into_iter()
                .map(|(o, items)| OrderSummaryData::from_db(o, items))
                .collect(),
            logged: true,
            ..Default::default()
        }
    }
}
