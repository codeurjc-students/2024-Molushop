//! Productos, sus variaciones, identificadores e imágenes, y las filas de
//! las consultas a mano de tarjetas y fichas (`*Row`).

use bigdecimal::BigDecimal;
use diesel::pg::sql_types::Jsonb;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Json, Nullable, Numeric, SmallInt, Text, Uuid as SqlUuid};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use crate::schema::*;

#[derive(Queryable, Debug, Deserialize,Serialize,Selectable)]
#[diesel(table_name = products)]
pub struct Products {
    pub id: Uuid,
    pub code : String,
    pub name: String,
    pub description: String,
    pub brand: String,
    pub status: i16,
    pub specs: Value,
    pub variations: Option<Value>,
    pub variation_titles: Option<Value>,
    pub images: Option<Value>,
    pub published: bool,    
}

/// El formulario del alta de producto (no es una tabla).
#[derive(Deserialize, Debug)]
pub struct ProductForm{
    pub code : String,
    pub name: String,
    pub description: String,
    pub brand: String,
    /// Opcionales: sin ningún campo de spec relleno, json-enc-custom no manda
    /// la clave y queda `Null` (`clean_specs` lo guarda como `{}`).
    #[serde(default)]
    pub specs: Value,
    pub variations: Option<Value>,
    pub images: Option<Value>,
    pub price: BigDecimal,
}

#[derive(Insertable)]
#[diesel(table_name = products)]
pub struct NewProduct<'a> {
    pub id: &'a Uuid,
    pub code : &'a str,
    pub name: &'a str,
    pub description: &'a str,
    pub brand: &'a str,
    pub specs: &'a Value,
    pub variations:  Option<&'a Value>,
    pub variation_titles:  Option<&'a Value>,
    pub images:Option<&'a Value>,
}

#[derive(Queryable, Debug, Deserialize,Serialize,Selectable)]
#[diesel(table_name = product_variations)]
pub struct ProductVariation {
    pub id: Uuid,
    pub product_id : Uuid,
    pub identifiers: Option<Value>,
    pub sku: Option<String>,
    pub attributes: Option<Value>,
    pub status: i16,
    pub stock: i32,
    pub images: Option<Value>,
    /// La tienda de su producto (la pone un trigger al insertar): el SKU es
    /// único por tienda.
    pub store_id: Option<Uuid>,
} 

#[derive(Insertable)]
#[diesel(table_name = product_variations)]
pub struct NewProductVariation<'a> {
    pub id: &'a Uuid,
    pub product_id : &'a Uuid,
    pub attributes: Option<&'a Value>,
}

#[derive(Queryable, Debug)]
#[diesel(table_name = product_variations_identifiers)]
pub struct IdentifierVariation {
    pub id: i32,
    pub product_variation_id: Uuid,
    pub identifier: String,
    pub value: String,
}

#[derive(Insertable)]
#[diesel(table_name = images_product)]
pub struct NewProductImage {
    pub id:Uuid,
    pub product_id:Uuid,
    pub image_url:String,
    pub is_main:bool,
    pub display_order:i32,
}

#[derive(Insertable)]
#[diesel(table_name = product_variations_identifiers)]
pub struct NewIdentifierVariation<'a> {
    pub product_variation_id:&'a Uuid,
    pub identifier:&'a str,
    pub value: &'a str,
}

#[derive(QueryableByName, Serialize, Deserialize, Debug, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProductCardRow {
    #[diesel(sql_type = SqlUuid)]
    pub id: Uuid,
    #[diesel(sql_type = Nullable<Text>)]
    pub name: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub brand: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub image_url: Option<String>,
    #[diesel(sql_type = Nullable<Numeric>)]
    pub price: Option<BigDecimal>,
    #[diesel(sql_type = Nullable<Text>)]
    pub currency: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub store_name: Option<String>,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    pub store_id: Option<Uuid>,
    /// Media de las opiniones, 0 si no hay; al vuelo, como en RatingSummaryRow.
    #[diesel(sql_type = diesel::sql_types::Double)]
    pub rating_average: f64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub rating_count: i64,
}
#[derive(QueryableByName, Serialize, Deserialize, Debug, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProductWithImagesRow{
    #[diesel(sql_type = SqlUuid)]
    pub id: Uuid,
    #[diesel(sql_type = Nullable<Text>)]
    pub code: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub name: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub description: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub brand: Option<String>,
    #[diesel(sql_type = Nullable<SmallInt>)]
    pub status: Option<i16>,
    #[diesel(sql_type = Nullable<Jsonb>)]
    pub specs: Option<Value>,
    #[diesel(sql_type = Nullable<Jsonb>)]
    pub variations: Option<Value>,
    #[diesel(sql_type = Nullable<Jsonb>)]
    pub variation_titles: Option<Value>,
    #[diesel(sql_type = Nullable<Jsonb>)]
    pub images: Option<Value>,
    #[diesel(sql_type = Nullable<Bool>)]
    pub published: Option<bool>,
    #[diesel(sql_type = Nullable<Json>)]
    pub images_product: Option<Value>
}

#[derive(QueryableByName, Serialize, Deserialize, Debug, Clone)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProductForPageRow{
    #[diesel(sql_type = SqlUuid)]
    pub id: Uuid,
    #[diesel(sql_type = Nullable<Text>)]
    pub name: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub brand: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub description: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub currency: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub store_name: Option<String>,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    pub store_id: Option<Uuid>,
    #[diesel(sql_type = Nullable<Jsonb>)]
    pub images_product: Option<Value>,
    #[diesel(sql_type = Nullable<Jsonb>)]
    pub variant_map: Option<Value>,
    #[diesel(sql_type = Nullable<Jsonb>)]
    pub variations_with_stock_status: Option<Value>,
    /// Media de las opiniones, 0 si no hay; al vuelo, como en RatingSummaryRow.
    #[diesel(sql_type = diesel::sql_types::Double)]
    pub rating_average: f64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub rating_count: i64
}
