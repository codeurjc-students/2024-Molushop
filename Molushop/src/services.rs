

use diesel::dsl::exists;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::dsl::select;
use diesel::dsl::sum;
use diesel::sql_query;
use diesel::sql_types::Bool;
use diesel::prelude::QueryDsl;
use dotenvy::dotenv;
use std::env;
use crate::models::*;
use bigdecimal::BigDecimal;
use uuid::Uuid;
use diesel::result::Error;
use diesel::{insert_into,update};
use chrono::NaiveDate;
use chrono::prelude::*;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}


pub fn insert_data_test() -> bool {
    use crate::schema::base_user::dsl::*;
    let other_id= Uuid::new_v4();
    let newUser = NewBaseUser{
        id: &other_id,
        name: "Test",
        lastname: "Test",
        email: "moluxo@hotmail.com",
        password: "tuAbuelaE",
        hash: "xdd",
        birthdate: &NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
    }; 

    let connection = &mut establish_connection();
    let result= insert_into(base_user).values(newUser).execute(connection);
    match result {
        Ok(num) => {
            println!("Data inserted {}",num);
            true
        },
        Err(_) => {
            println!("Error inserting data");
            false 
        }
    }

}


pub fn modify_data_test() -> bool {
    use crate::schema::base_user::dsl::*;
    let connection = &mut establish_connection();
    let result = update(base_user).set((email.eq("otro@gmail.com"),name.eq("GIGA"),modified.eq(Utc::now().naive_utc()))).execute(connection);
    match result {
        Ok(num) => {
            println!("Data modified {}",num);
            true
        },
        Err(_) => {
            println!("Error modifying data");
            false 
        }
    }
}

pub fn obtain_base_categories() -> Vec<Category> {
    use crate::schema::category::dsl::*;
    let connection = &mut establish_connection();
    let results = category.filter(depth.eq(0)).load::<Category>(connection).expect("Error loading categories");
    results
}

pub fn obtain_categories_children(id_category:&String) -> Vec<Category> {
    use crate::schema::category::dsl::*;
    let connection = &mut establish_connection();
    let results = category.filter(parent.eq(id_category)).load::<Category>(connection).expect("Error loading categories");
    results
}