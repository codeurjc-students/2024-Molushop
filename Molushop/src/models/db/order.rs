//! Pedidos, sus líneas y la dirección de envío.

use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;
use crate::schema::*;

#[derive(Queryable, Debug)]
#[diesel(table_name = customer_address)]
pub struct CustomerAddress {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub street1: String,
    pub street2: Option<String>,
    pub postal_code: String,
    pub city: String,
    pub province: String,
}

/// La dirección se inserta en el mismo POST que confirma el pedido, para que no
/// queden filas huérfanas si el pedido se cae. Ver la migración
/// 2026-09-01-192454_orders: `orders` guarda además una COPIA de estos campos.
#[derive(Insertable)]
#[diesel(table_name = customer_address)]
pub struct NewCustomerAddress<'a> {
    pub id: &'a Uuid,
    pub customer_id: &'a Uuid,
    pub street1: &'a str,
    pub street2: Option<&'a str>,
    pub postal_code: &'a str,
    pub city: &'a str,
    pub province: &'a str,
}

// El pedido guarda copia de lo que se vendió y de a dónde se envió: los
// importes son los cobrados y no se recalculan. Ver la migración
// 2026-09-01-192454_orders.
#[derive(Queryable, Debug, Identifiable)]
#[diesel(table_name = orders)]
pub struct Order {
    pub id: Uuid,
    pub order_number: i64,
    pub user_id: Uuid,
    pub cart_id: Option<Uuid>,
    pub status: i16,
    pub currency: String,
    pub subtotal: BigDecimal,
    pub discount_total: BigDecimal,
    pub shipping_total: BigDecimal,
    pub total_amount: BigDecimal,
    pub shipping_address_id: Option<Uuid>,
    pub ship_street1: String,
    pub ship_street2: Option<String>,
    pub ship_postal_code: String,
    pub ship_city: String,
    pub ship_province: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// Sin order_number, created_at ni updated_at: los pone la base de datos.
#[derive(Insertable)]
#[diesel(table_name = orders)]
pub struct NewOrder<'a> {
    pub id: &'a Uuid,
    pub user_id: &'a Uuid,
    pub cart_id: Option<&'a Uuid>,
    pub status: i16,
    pub currency: &'a str,
    pub subtotal: &'a BigDecimal,
    pub discount_total: &'a BigDecimal,
    pub shipping_total: &'a BigDecimal,
    pub total_amount: &'a BigDecimal,
    pub shipping_address_id: Option<&'a Uuid>,
    pub ship_street1: &'a str,
    pub ship_street2: Option<&'a str>,
    pub ship_postal_code: &'a str,
    pub ship_city: &'a str,
    pub ship_province: &'a str,
}

#[derive(Queryable, Debug, Identifiable)]
#[diesel(table_name = order_items)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_var_id: Uuid,
    pub product_name: String,
    pub variation_label: Option<String>,
    pub sku: Option<String>,
    pub image_url: Option<String>,
    pub store_name: Option<String>,
    pub unit_price: BigDecimal,
    pub currency: String,
    pub quantity: i32,
    pub line_total: BigDecimal,
    /// Tienda de la línea, para "pedidos de mi tienda". `None` si la tienda se
    /// borró o la línea es anterior y su producto ya no tiene tienda.
    pub store_id: Option<Uuid>,
    /// Estado de envío de ESTA línea (mismos valores que `orders.status`).
    pub status: i16,
}

#[derive(Insertable)]
#[diesel(table_name = order_items)]
pub struct NewOrderItem<'a> {
    pub id: &'a Uuid,
    pub order_id: &'a Uuid,
    pub product_var_id: &'a Uuid,
    pub product_name: &'a str,
    pub variation_label: Option<&'a str>,
    pub sku: Option<&'a str>,
    pub image_url: Option<&'a str>,
    pub store_name: Option<&'a str>,
    pub unit_price: &'a BigDecimal,
    pub currency: &'a str,
    pub quantity: i32,
    pub line_total: &'a BigDecimal,
    pub store_id: Option<&'a Uuid>,
    pub status: i16,
}
