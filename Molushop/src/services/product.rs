use super::servicesX::establish_connection;
use diesel::result::Error;
use uuid::Uuid;
//use crate::models::models_x::{Category,NewBaseUser,NewProduct,Products,ProductForm};

use diesel::prelude::*;
//use diesel::dsl::select;
//use diesel::dsl::sum;
//use diesel::sql_query;
//use diesel::sql_types::Bool;
//use diesel::prelude::QueryDsl;
//use diesel::sql_types::Text;
//use dotenvy::dotenv;
//use std::env;
//use crate::models::get_product::{GetProductForm,Variation};
//use bigdecimal::BigDecimal;
//use diesel::{insert_into,update};
//use chrono::NaiveDate;
//use chrono::prelude::*;
//use serde_json::Value;
//use serde_json::json;

pub fn get_name(id_product:&Uuid) -> Result<String, Error>{
    use crate::schema::products::dsl::*;
    let connection = &mut establish_connection();
    let result = products.filter(id.eq(id_product)).select(name).first::<String>(connection);
    //let result = products.filter(id.eq(id_product)).first::<Products>(connection);
    result
}