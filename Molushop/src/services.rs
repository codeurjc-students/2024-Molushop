

use diesel::dsl::exists;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::dsl::select;
use diesel::dsl::sum;
use diesel::sql_query;
use diesel::sql_types::Bool;
use diesel::prelude::QueryDsl;
use diesel::sql_types::Text;
use dotenvy::dotenv;
use std::env;
use crate::models::*;
use bigdecimal::BigDecimal;
use uuid::Uuid;
use diesel::result::Error;
use diesel::{insert_into,update};
use chrono::NaiveDate;
use chrono::prelude::*;
use serde_json::Value;

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

pub fn obtain_base_categories() -> Result<Vec<Category>,Error> {
    use crate::schema::category::dsl::*;
    let connection = &mut establish_connection();
    let results = category.filter(depth.eq(0)).load::<Category>(connection);
    results
}

pub fn obtain_categories_children(id_category:&String) -> Result<Vec<Category>,Error> {
    use crate::schema::category::dsl::*;
    let connection = &mut establish_connection();
    let results = category.filter(parent.eq(id_category)).load::<Category>(connection);
    results
}

pub fn obtain_category(id_category:&String) -> Category {
    use crate::schema::category::dsl::*;
    let connection = &mut establish_connection();
    let result = category.filter(id.eq(id_category)).first::<Category>(connection).expect("Error loading category");
    result
}

pub fn obtain_ancestors(id_category:&String) -> Result<Vec<Category>,Error> {
    use crate::models::Category;
    let connection = &mut establish_connection();

    println!("ID: {}",id_category);
    //el placeholder $1 es para evitar sql injection
    let ancestors:Result<Vec<Category>, Error>  = sql_query("WITH RECURSIVE Ancestors AS (SELECT id, name, parent, depth, base_specs, is_parent FROM Category WHERE id = $1 UNION ALL  SELECT c.id, c.name, c.parent, c.depth, c.base_specs, c.is_parent FROM Category c INNER JOIN Ancestors a ON c.id = a.parent) SELECT * FROM Ancestors order by depth")
    .bind::<Text,_>(id_category.to_string()).get_results(connection);
    
    ancestors
}

pub fn obtain_base_specs(id_category:&String) -> Result<Vec<Option<Value>>,Error> {
   
    use crate::schema::category::dsl::*;
    let connection = &mut establish_connection();
    let result = category.filter(id.eq(id_category)).select(base_specs).load::<Option<Value>>(connection);
    result
    
}

pub fn insert_new_product(form:&ProductForm) -> Result<usize, Error> {
    use crate::schema::products::dsl::*;
    let connection = &mut establish_connection();
    let new_product = NewProduct{
        id: &Uuid::new_v4(),
        code: &form.code,
        name: &form.name,
        description: &form.description,
        brand: &form.brand,
        specs: &form.specs,
        variations: form.variations.as_ref(),
        images: form.images.as_ref(),
    };
    let result = insert_into(products).values(new_product).execute(connection);
    result
}

