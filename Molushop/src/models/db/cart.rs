//! El carrito y sus líneas.

use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Integer, Nullable, Numeric, Text, Uuid as SqlUuid};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::schema::*;

#[derive(Queryable, Debug, Identifiable)]
#[diesel(table_name = carts)]
pub struct Cart {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub status: i16,
}

#[derive(Insertable)]
#[diesel(table_name = carts)]
pub struct NewCart<'a> {
    pub id: &'a Uuid,
    pub user_id: &'a Uuid,
    pub status: i16,
}

#[derive(Queryable, Debug, Identifiable)]
#[diesel(table_name = cart_products)]
pub struct CartProduct {
    pub id: Uuid,
    pub cart_id: Uuid,
    pub product_var_id: Uuid,
    pub quantity: i32,
    pub price_at_time_of_addition: Option<BigDecimal>,
    pub added_at: NaiveDateTime,
}

#[derive(QueryableByName, Serialize, Deserialize, Debug, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CartItemRow{
    #[diesel(sql_type = SqlUuid)]
    pub id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    pub product_var_id: Uuid,
    #[diesel(sql_type = SqlUuid)]
    pub product_id: Uuid,
    #[diesel(sql_type = Nullable<Text>)]
    pub name: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub variation_label: Option<String>,
    /// Referencia de la variación. El carrito no la pinta; está aquí porque el
    /// snapshot de `order_items` la guarda.
    #[diesel(sql_type = Nullable<Text>)]
    pub sku: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub image_url: Option<String>,
    #[diesel(sql_type = Nullable<Numeric>)]
    pub price: Option<BigDecimal>,
    #[diesel(sql_type = Nullable<Text>)]
    pub currency: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub store_name: Option<String>,
    /// La tienda de `store_name`. El carrito no la pinta; está aquí porque la
    /// línea del pedido la guarda.
    #[diesel(sql_type = Nullable<SqlUuid>)]
    pub store_id: Option<Uuid>,
    #[diesel(sql_type = Integer)]
    pub quantity: i32,
    #[diesel(sql_type = Integer)]
    pub stock: i32,
    /// Producto y variación activos. Un producto eliminado o inactivo, o una
    /// variación desactivada, ya no se venden aunque sigan en el carrito.
    #[diesel(sql_type = Bool)]
    pub on_sale: bool,
    #[diesel(sql_type = Nullable<Numeric>)]
    pub subtotal: Option<BigDecimal>
}
