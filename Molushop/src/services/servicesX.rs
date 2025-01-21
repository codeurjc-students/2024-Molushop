

use diesel::dsl::exists;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel_async::{RunQueryDsl, AsyncConnection, AsyncPgConnection};
use diesel::dsl::select;
use diesel::dsl::sum;
use diesel::sql_query;
use diesel::sql_types::Bool;
use diesel::prelude::QueryDsl;
use diesel::sql_types::Text;
use dotenvy::dotenv;
use std::env;
use crate::models::models_x::{Category,NewBaseUser,NewProduct,Products,ProductForm,NewProductVariation1,ProductVariation};
use crate::models::get_product::{GetProductForm,Variation};
use crate::models::product_variation;
use bigdecimal::BigDecimal;
use uuid::Uuid;
use diesel::result::Error;
use diesel::{insert_into,update}; 
use chrono::NaiveDate;
use chrono::prelude::*;
use serde_json::Value;
use serde_json::json;

use serde::{Deserialize, Serialize};

use diesel_async::pooled_connection::deadpool::Pool;
//use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}



pub fn combine<T: Clone>(lists: Vec<Vec<T>>) -> Vec<Vec<T>> {
    if lists.is_empty() {
        return vec![vec![]];
    }

    let mut result = Vec::new();
    let first_list = &lists[0];
    let rest_lists = &lists[1..];

    for item in first_list {
        for combination in combine(rest_lists.to_vec()) {
            let mut new_combination = vec![item.clone()];
            new_combination.extend(combination);
            result.push(new_combination);
        }
    }

    result
}

pub fn combine_tail_recursive<T: Clone>(lists: Vec<Vec<T>>) -> Vec<Vec<T>> {
    fn helper<T: Clone>(lists: &[Vec<T>], acc: Vec<Vec<T>>) -> Vec<Vec<T>> {
        if lists.is_empty() {
            return acc;
        }

        let first_list = &lists[0];
        let rest_lists = &lists[1..];

        let mut new_acc = Vec::new();
        for combination in acc { //tiene un elemento vacio
            for item in first_list {
                let mut new_combination = combination.clone();
                new_combination.push(item.clone());
                new_acc.push(new_combination);
            }
        }

        helper(rest_lists, new_acc)
    }

    helper(&lists, vec![vec![]])
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
    /* 
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
    */
    true

}


pub fn modify_data_test() -> bool {
    use crate::schema::base_user::dsl::*;
    let connection = &mut establish_connection();
    /*let result = update(base_user).set((email.eq("otro@gmail.com"),name.eq("GIGA"),modified.eq(Utc::now().naive_utc()))).execute(connection);
    match result {
        Ok(num) => {
            println!("Data modified {}",num);
            true
        },
        Err(_) => {
            println!("Error modifying data");
            false 
        }
    }*/
    true
}

pub async fn obtain_base_categories(pool:&DbPool) -> Result<Vec<Category>,Error> {
    
    use crate::schema::category::dsl::*;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    let results = category.filter(depth.eq(0)).load::<Category>(connection).await;
    results
}

pub async fn obtain_categories_children(id_category:&String, pool: &DbPool) -> Result<Vec<Category>,Error> {
    use crate::schema::category::dsl::*;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    //let connection = &mut establish_connection();
    let results = category.filter(parent.eq(id_category)).load::<Category>(connection).await;
    results
}

pub async fn obtain_category(id_category:&String, pool:&DbPool) -> Category {
    use crate::schema::category::dsl::*;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    let result = category.filter(id.eq(id_category)).first::<Category>(connection).await.expect("Error loading category");
    result
}

pub async fn obtain_ancestors(id_category:&String, pool: &DbPool) -> Result<Vec<Category>,Error> {
    use crate::models::models_x::Category;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    println!("ID: {}",id_category);
    //el placeholder $1 es para evitar sql injection
    let ancestors:Result<Vec<Category>, Error>  = sql_query("WITH RECURSIVE Ancestors AS (SELECT id, name, parent, depth, base_specs, is_parent FROM Category WHERE id = $1 UNION ALL  SELECT c.id, c.name, c.parent, c.depth, c.base_specs, c.is_parent FROM Category c INNER JOIN Ancestors a ON c.id = a.parent) SELECT * FROM Ancestors order by depth")
    .bind::<Text,_>(id_category.to_string()).get_results(connection).await;
    
    ancestors
}

pub async fn obtain_base_specs(id_category:&String, pool:&DbPool) -> Result<Vec<Option<Value>>,Error> {
   
    use crate::schema::category::dsl::*;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    let result = category.filter(id.eq(id_category)).select(base_specs).load::<Option<Value>>(connection).await;
    result
    
}
pub async fn insert_new_product_complete(id_seler:Uuid,form:&ProductForm,category_id:&String, pool:&DbPool) -> Result<Uuid, Error> {
    /* 
    let result_new_product = insert_new_product(form,pool).await;
    match result_new_product{
        Ok(id_product) => {
            let result2 = insert_product_seller(&id_seler,&id_product,pool).await;
            match result2{
                Ok(_) => {
                    let result3 = insert_product_category(&id_product,&category_id,pool).await;
                    match result3{
                        Ok(_) => {
                            Ok(id_product)
                        },
                            Err(e) => {
                                return Err(e);
                            }
                        }
                },
                Err(e) => {
                    return Err(e);
                }
            }
        },
        Err(e) => {
            return Err(e);
        }
    }
    */
    let id_product = insert_new_product(form,pool).await?;
    insert_product_seller(&id_seler,&id_product,pool).await?;
    insert_product_category(&id_product,&category_id,pool).await?;
    Ok(id_product)
}

pub async fn insert_product_category(id_product:&Uuid, id_category:&String, pool:&DbPool) -> Result<usize,Error> {
    use crate::schema::category_product::dsl::*;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    let result = insert_into(category_product).values((category_id.eq(id_category),product_id.eq(id_product))).execute(connection).await;
    result
}


pub async fn insert_product_seller(id_seler:&Uuid, id_product:&Uuid, pool:&DbPool) -> Result<usize,Error> {
    use crate::schema::product_seller::dsl::*;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    let result = insert_into(product_seller).values((product_id.eq(id_product),seller_id.eq(id_seler))).execute(connection).await;
    result
}

#[derive(Deserialize, Serialize, Debug)]
struct FormTitulos {
    titulos: Vec<String>
}

pub async fn insert_new_product(form:&ProductForm,pool:&DbPool) -> Result<Uuid, Error> {
    use crate::schema::products::dsl::*;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    let new_id = Uuid::new_v4();
    //metodo para obtener los nombres de los titulos de las variaciones
    //obtenern ese jsson b y obtener los nombres de las keys
    let titles:Option<Value> = match form.variations.clone(){
        Some(a)=>{
            Some(json!({
                "titulos": a.as_array()
                    .unwrap()
                    .iter()
                    .map(|item| item["name"].as_str().unwrap())
                    .collect::<Vec<&str>>()
                }))
        },
        None=>None

    };
    

    let new_product = NewProduct{
        id: &new_id,
        code: &form.code,
        name: &form.name,
        description: &form.description,
        brand: &form.brand,
        specs: &form.specs,
        variations: form.variations.as_ref(),
        variation_titles: titles.as_ref(), //de momento, cambiar
        images: form.images.as_ref(),
    };
    let result = insert_into(products).values(new_product).execute(connection).await;
    match result {
        Ok(_) => {
            //println!("Data inserted {}",num);
            Ok(new_id)
        },
        Err(e) => {
            //println!("Error inserting data: {}",e);
            Err(e)
        }
    }
}

pub async fn get_product(id_product:&Uuid, pool:&DbPool) -> Result<Products,Error> {
    use crate::schema::products::dsl::*;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    let result = products.filter(id.eq(id_product)).first::<Products>(connection).await;
    result
}

use crate::models::product_variation::VariationValue;

 pub async fn insert_product_variations(prod_id:&Uuid,combinations: Vec<Vec<VariationValue>>,pool:&DbPool) -> Result<usize,Error>{
    use crate::schema::product_variations::dsl::*;
    //let connection = &mut establish_connection();
    let connection= &mut pool.get().await.unwrap();
    for combination in combinations{
        let json_attributes:Value = serde_json::to_value(&combination).unwrap();
        let new_variation = NewProductVariation1{
            id: &Uuid::new_v4(),
            product_id: prod_id,
            attributes: Some(&json_attributes),
        };
        let result = insert_into(product_variations).values(new_variation).execute(connection).await;
        match result {
            Ok(num) => {
                println!("Data inserted {}",num);
            },
            Err(e) => {
                println!("Error inserting data: {}",e);
                return Err(e);
            }
        }
    }
    Ok(1)
 }

 pub async fn insert_product_variation(prod_id:&Uuid, pool:&DbPool) -> Result<usize,Error>{
    use crate::schema::product_variations::dsl::*;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    let new_variation = NewProductVariation1{
        id: &Uuid::new_v4(),
        product_id: prod_id,
        attributes: None,
    };
    let result = insert_into(product_variations).values(new_variation).execute(connection).await;
    result
 }

pub async fn get_seller_products(user_id:&Uuid,pool: &DbPool) -> Result<Vec<Products>,Error>{
    use crate::schema::product_seller::dsl::*;
    use crate::schema::products::dsl::*;
    //use crate::schema::products;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    let results = products
        .inner_join(product_seller.on(id.eq(product_id)))
        .filter(seller_id.eq(user_id))
        .select(Products::as_select())
        .load::<Products>(connection).await;
    results
}

pub async fn get_product_categories(id_product:&Uuid,pool:&DbPool) -> Result<Vec<Category>,Error> {
    use crate::schema::category_product::dsl::*;
    use crate::schema::category::dsl::*;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    let results = category
        .inner_join(category_product.on(id.eq(category_id)))
        .filter(product_id.eq(id_product))
        .select(Category::as_select())
        .load::<Category>(connection).await;
    results
}

pub async fn get_product_variations(id_product:&Uuid,pool:&DbPool) -> Result<Vec<ProductVariation>,Error> {
    use crate::schema::product_variations::dsl::*;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    let results = product_variations
        .filter(product_id.eq(id_product))
        .load::<ProductVariation>(connection).await;
    results
}

pub async fn delete_seller_product(prod_id:&Uuid,sell_id:&Uuid,pool:&DbPool) -> Result<usize,Error> {
    use crate::schema::product_seller::dsl::*;
    
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    let valor= diesel::delete(product_seller
        .filter(product_id.eq(prod_id).and(seller_id.eq(sell_id)))
    ).execute(connection).await?;
    //borrar producto

    //borrar variaciones

    //en este contexto es borrar el producto entero, en otros puede variar 
    Ok(valor)
}

pub async fn delete_product_complete(id_producto:&Uuid,pool: &DbPool) -> Result<usize,Error>{
    //TODO
    Ok(1)
}