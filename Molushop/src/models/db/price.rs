//! Precios y descuentos de las variaciones.

use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;
use crate::schema::*;

#[derive(Insertable)]
#[diesel(table_name = prices)]
pub struct NewPrice<'a> {
    pub variation_id:&'a Uuid,
    pub price:&'a BigDecimal,
    pub currency:&'a str,
    pub start_date:&'a NaiveDateTime,
}

//No meter mas derive en esas cosas
#[derive(Queryable, Debug)]
#[diesel(table_name = prices)]
pub struct Price {
    pub id: i32,
    pub variation_id: Uuid,
    pub price: BigDecimal,
    pub currency: String,
    pub start_date: NaiveDateTime
}

#[derive(Queryable, Debug, Identifiable)]
#[diesel(table_name = discounts)]
pub struct Discounts {
    pub id:i32,
    pub variation_id:Uuid,
    pub discount_type:i16,
    pub percentage: Option<BigDecimal>,
    pub quantity: Option<i32>,
    pub discount_value:BigDecimal,
    pub currency: String,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = discounts)]
pub struct NewDiscount<'a> {
    pub variation_id:&'a Uuid,
    pub discount_type:&'a i16,
    pub percentage: Option<&'a BigDecimal>,
    pub quantity: Option<&'a i32>,
    pub discount_value: &'a BigDecimal,
    pub currency: &'a str,
    pub start_date: Option<&'a NaiveDateTime>,
    pub end_date: Option<&'a NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = discount_history)]
pub struct NewDiscountHistory<'a> {
    pub variation_id:&'a Uuid,
    pub discount_type:&'a i16,
    pub percentage: Option<&'a BigDecimal>,
    pub quantity: Option<&'a i32>,
    pub discount_value: &'a BigDecimal,
    pub currency: &'a str,
    pub start_date: Option<&'a NaiveDateTime>,
    pub end_date: Option<&'a NaiveDateTime>,
    pub created_at: &'a NaiveDateTime
}
