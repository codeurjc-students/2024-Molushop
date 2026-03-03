

//use diesel::RunQueryDsl;
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
use crate::models;
use crate::models::models_x::{ProductForPage1,ProductWithImages1,Discounts,Category,NewBaseUser,NewProduct,Products,ProductForm,NewProductVariation1,ProductVariation,UserAuth,NewUserSession,ProductCard1};
use crate::models::get_product::{GetProductForm,Variation};
use crate::models::product_variation;
use crate::schema::base_user::password;
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
        username:"Moluxo",
        name: "Test",
        lastname: "Test",
        email: "moluxo@hotmail.com",
        password: "tuAbuelaE",
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

 pub async fn insert_product_variations(prod_id:&Uuid,combinations: Vec<Vec<VariationValue>>,pool:&DbPool) -> Result<Vec<Uuid>,Error>{
    use crate::schema::product_variations::dsl::*;
    //let connection = &mut establish_connection();
    let mut new_ids:Vec<Uuid>= Vec::new();
    let connection= &mut pool.get().await.unwrap();
    for combination in combinations{
        let new_id = Uuid::new_v4();
        let json_attributes:Value = serde_json::to_value(&combination).unwrap();
        let new_variation = NewProductVariation1{
            id: &new_id,
            product_id: prod_id,
            attributes: Some(&json_attributes),
        };
        new_ids.push(new_id);
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
    Ok(new_ids)
 }

 pub async fn insert_product_variation(prod_id:&Uuid, pool:&DbPool) -> Result<Uuid,Error>{
    use crate::schema::product_variations::dsl::*;
    //let connection = &mut establish_connection();
    let new_id = Uuid::new_v4();
    let connection = &mut pool.get().await.unwrap();
    let new_variation = NewProductVariation1{
        id: &new_id,
        product_id: prod_id,
        attributes: None,
    };
    let result = insert_into(product_variations).values(new_variation).execute(connection).await;
    match result{
        Ok(_) => {
            Ok(new_id)
        },
        Err(e) => {
            Err(e)
        }
    }
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

pub async fn get_product_seller(user_id:&Uuid,prod_id:&Uuid,pool: &DbPool) -> Result<Products,Error>{
    use crate::schema::product_seller::dsl::*;
    use crate::schema::products::dsl::*;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    let result = products
        .inner_join(product_seller.on(id.eq(product_id)))
        .filter(seller_id.eq(user_id).and(id.eq(prod_id)))
        .select(Products::as_select())
        .first::<Products>(connection).await;
    result
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

pub async fn get_product_variation(id_variation:&Uuid,pool:&DbPool) -> Result<ProductVariation,Error> {
    use crate::schema::product_variations::dsl::*;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    let result = product_variations.filter(id.eq(id_variation)).first::<ProductVariation>(connection).await;
    result
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

use crate::controllers::components::edit_product_controller::FormGeneral;

pub async fn edit_product_general(id_product:&Uuid,form:FormGeneral,pool:&DbPool) -> Result<usize,Error> {
    use crate::schema::products::dsl::*;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();

    let result = update(products.filter(id.eq(id_product)))
        .set((
            name.eq(form.name),
            description.eq(form.description),
            brand.eq(form.brand),
            status.eq(form.status)
        ))
        .execute(connection).await;
    result
}

pub async fn update_image(id_producto:&Uuid,tipo:&String,image_url:&String,pool:&DbPool) -> Result<usize,Error> {
    //use crate::schema::products::dsl::*;
    //let connection = &mut establish_connection();
    let connection = &mut pool.get().await.unwrap();
    //let other_id = "2fcddbef-1c29-4601-8879-8671bc77160b";
    //let tipo = "principal";
    //let url = "https://ejemplo.com/principal.jpg";

    let result = sql_query(r#"
    UPDATE products 
    SET images = jsonb_set(
    images,
    '{images}',
    (images->'images') || json_build_array(
        json_build_object(
            'tipo', $2,
            'url', $3
        )
    )::jsonb
    )
    WHERE id = $1
    "#).bind::<diesel::sql_types::Uuid,_>(id_producto)
    .bind::<diesel::sql_types::Text,_>(tipo)
    .bind::<diesel::sql_types::Text,_>(image_url)
    .execute(connection).await;
    result
}

use crate::controllers::components::edit_product_controller::ImageData;
use crate::controllers::components::edit_product_controller::ImageData2;

pub async fn update_multiple_images(id_producto: &Uuid, images: &[ImageData], pool: &DbPool) -> Result<usize, Error> {
    let connection = &mut pool.get().await.unwrap();
    
    // Convertir el array de ImageData a un array JSON
    let images_json: Vec<serde_json::Value> = images
        .iter()
        .map(|img| json!({
            "tipo": img.tipo,
            "url": img.url
        }))
        .collect();
    
    let json_array = serde_json::to_string(&images_json).unwrap();
    
    let result = sql_query(r#"
    UPDATE products 
    SET images = jsonb_set(
        images,
        '{images}',
        (images->'images') || $2::jsonb
    )
    WHERE id = $1
    "#)
    .bind::<diesel::sql_types::Uuid, _>(id_producto)
    .bind::<diesel::sql_types::Text, _>(json_array)
    .execute(connection)
    .await;
    
    result
}
use crate::models::models_x::NewImageProduct;
use crate::models::models_x::NewImageProduct2;
pub async fn update_multiple_images_2(image_products:Vec<NewImageProduct2>, pool: &DbPool) -> Result<usize, Error> {
    let connection = &mut pool.get().await.unwrap();
    use crate::schema::images_product::dsl::*;
    let result = insert_into(images_product).values(image_products).execute(connection).await;
    match result{
        Ok(num)=>{
            return Ok(num)
        },
        Err(e)=>{
            return Err(e);
        }
    }
}


//pub async fn get_product_cards() -> Rersult<>

pub async fn delete_all_images(id_producto: &Uuid, pool: &DbPool) -> Result<usize, Error> {
    let connection = &mut pool.get().await.unwrap();
    
    let result = sql_query(r#"
    UPDATE products 
    SET images = jsonb_set(
        images,
        '{images}',
        '[]'::jsonb
    )
    WHERE id = $1
    "#)
    .bind::<diesel::sql_types::Uuid, _>(id_producto)
    .execute(connection)
    .await;
    
    result
}
/* 
pub async fn get_images(id_producto: &Uuid, pool: &DbPool) -> Result<Vec<ImageData>, Error> {
    let connection = &mut pool.get().await.unwrap();
    
    let result = sql_query(r#"
    SELECT jsonb_array_elements(images->'images') AS image
    FROM products
    WHERE id = $1
    "#)
    .bind::<diesel::sql_types::Uuid, _>(id_producto)
    .load::<(serde_json::Value,)>(connection)
    .await?;
    
    let images: Vec<ImageData> = result
        .iter()
        .map(|(row,)| {
            let image = row.as_object().unwrap();
            ImageData {
                tipo: image["tipo"].as_str().unwrap().to_string(),
                url: image["url"].as_str().unwrap().to_string(),
            }
        })
        .collect();
    
    Ok(images)
}
*/
use crate::models::models_x::{NewPrice,Price};
use diesel::upsert::excluded;
pub async fn insert_prices_variations(
    variations_ids: &[Uuid],
    price_insert: &BigDecimal,
    currency_insert: &str,
    pool: &DbPool 
) -> Result<usize, Error> {
    use crate::schema::prices::dsl::*;
    //crear los modelos de insercion de precio
    let connection = &mut pool.get().await.unwrap();
    let now = Utc::now().naive_utc();
    let new_prices:Vec<NewPrice> = variations_ids
        .iter()
        .map(|id_x| NewPrice{
            variation_id: id_x,
            price: price_insert,
            currency: currency_insert,
            start_date: &now
        })
        .collect();

     insert_into(prices)
        .values(&new_prices)
        .on_conflict((variation_id,currency))
        .do_update()
        .set((
            price.eq(excluded(price)),
            start_date.eq(excluded(start_date))
        ))
        .execute(connection)
        .await
}

pub async fn get_variation_price(var_id:&Uuid, curr:&String, pool: &DbPool) -> Result<Price,Error>{
    use crate::schema::prices::dsl::*;

    //obtener el precio actual, con el descuento?
    //obtener el descuento va aparte
    let connection = &mut pool.get().await.unwrap();
    let result = prices
        .filter(variation_id.eq(var_id))
        .filter(currency.eq(curr))
        .first::<Price>(connection)
        .await;
    result
}
use crate::models::models_x::{IdentifierVariation,NewIdentifierVariation};
pub async fn get_variation_identifiers(var_id:&Uuid, pool:&DbPool) -> Result<Vec<IdentifierVariation>,Error>{
    use crate::schema::product_variations_identifiers::dsl::*;
    let connection = &mut pool.get().await.unwrap();
    let result = product_variations_identifiers
        .filter(product_variation_id.eq(var_id))
        .load::<IdentifierVariation>(connection).await;
    result
}
 
pub async fn get_variations_with_prices(prod_id:&Uuid,curr:&String,pool: &DbPool) -> Result<Vec<(ProductVariation, Price)>, Error> {
    use crate::schema::product_variations::dsl as pv;
    use crate::schema::prices::dsl as p;
    
    let connection = &mut pool.get().await.unwrap();
    
    let results = pv::product_variations
        .inner_join(p::prices.on(pv::id.eq(p::variation_id)))
        .filter(p::currency.eq(curr))
        .filter(pv::product_id.eq(prod_id))
        .load::<(ProductVariation, Price)>(connection)
        .await?;
    
    Ok(results)
}

use crate::models::components::edit_product_variation_model::Identifier;
pub async fn set_variation_identifiers_antiguo(
    vec_identifiers:&[Identifier],
    var_id:&Uuid,
    pool: &DbPool
)-> Result<usize,Error>{
    //añadir o sobreescribir?
    let connection = &mut pool.get().await.unwrap();
    let insert_identifiers:Vec<NewIdentifierVariation> = vec_identifiers
        .iter()
        .map(|ident| NewIdentifierVariation{
            product_variation_id:&var_id,
            identifier:&ident.name,
            value:&ident.value
        })
        .collect();
    

    use crate::schema::product_variations_identifiers::dsl::*;
    insert_into(product_variations_identifiers)
        .values(&insert_identifiers)
        .on_conflict((product_variation_id,identifier))
        .do_update()
        .set(
            value.eq(excluded(value))
        )
        .execute(connection)
        .await
}
////NUEVO///
pub async fn set_variation_identifiers(
    vec_identifiers: &[Identifier],
    var_id: &Uuid,
    pool: &DbPool
) -> Result<usize, Error> {
    use crate::schema::product_variations_identifiers::dsl::*;
    let connection = &mut pool.get().await.unwrap();
    
    // Iniciamos una transacción para garantizar atomicidad
    connection.build_transaction()
        .run(|tx| Box::pin(async move {
            // 1. Primero: Obtener los identificadores actuales para esta variación
            let current_identifiers = product_variations_identifiers
                .filter(product_variation_id.eq(var_id))
                .select(identifier)
                .load::<String>(tx)
                .await?;
            
            // 2. Obtener los nombres de los nuevos identificadores
            let new_identifier_names: Vec<String> = vec_identifiers
                .iter()
                .map(|x_id| x_id.name.clone())
                .collect();
            
            // 3. Eliminar los identificadores que ya no están en la nueva lista
            let to_delete: Vec<String> = current_identifiers
                .into_iter()
                .filter(|y_id| !new_identifier_names.contains(y_id))
                .collect();
            
            if !to_delete.is_empty() {
                diesel::delete(product_variations_identifiers)
                    .filter(product_variation_id.eq(var_id))
                    .filter(identifier.eq_any(to_delete))
                    .execute(tx)
                    .await?;
            }
            
            // 4. Insertar/actualizar los nuevos identificadores
            let insert_identifiers: Vec<NewIdentifierVariation> = vec_identifiers
                .iter()
                .map(|ident| NewIdentifierVariation {
                    product_variation_id: var_id,
                    identifier: &ident.name,
                    value: &ident.value
                })
                .collect();
            
            let result = insert_into(product_variations_identifiers)
                .values(&insert_identifiers)
                .on_conflict((product_variation_id, identifier))
                .do_update()
                .set(value.eq(excluded(value)))
                .execute(tx)
                .await;
                
            result
        }))
        .await
}

pub async fn get_identifier_options_var(pool: &DbPool) -> Result<Vec<String>, Error> {
    use crate::schema::identifiers_var::dsl::*;
    
    let connection = &mut pool.get().await.unwrap();
    let results = identifiers_var
        .select(value)
        .order_by(value.asc())
        .load::<String>(connection)
        .await;
    
    results
}

pub async fn delete_variation_identifier(var_id:&Uuid,name:&String,pool:&DbPool)->Result<usize,Error>{
    Ok(99)
}

pub async fn update_variation_status(var_id:&Uuid,status_value:&i16,pool:&DbPool)-> Result<usize,Error>{
    use crate::schema::product_variations::dsl::*;

    let connection = &mut pool.get().await.unwrap();

    let result = update(product_variations)
        .filter(id.eq(var_id))
        .set(status.eq(status_value))
        .execute(connection)
        .await;
    result
    
}
use crate::models::components::edit_product_variation_model::PriceData;
use crate::models::models_x::{NewDiscount,NewDiscountHistory};
pub async fn update_price_variation(form:&PriceData, var_id:&Uuid,price_insert: &BigDecimal,currency_insert: &String,pool:&DbPool)-> Result<usize,Error>{
    use crate::schema::prices;
    use crate::schema::price_history; //::dsl::nombre
    use crate::schema::discounts::dsl::*;
    //use crate::schema::discount_history::dsl::*;
    use crate::schema::discounts;
    use crate::schema::discount_history;

    let connection = &mut pool.get().await.unwrap();

    connection.build_transaction()
        .run(|tx| Box::pin(async move{
            //Primero actualizar el precio 
            let now = chrono::Utc::now().naive_utc();

            //si el precio es 
            let result_price = update(prices::dsl::prices.filter(prices::dsl::variation_id.eq(&var_id).and(prices::dsl::currency.eq(&currency_insert))))
                .set((
                    prices::dsl::price.eq(&price_insert),
                    prices::dsl::start_date.eq(&now)
                ))
                .execute(tx).await?;

            let result_price_history= insert_into(price_history::dsl::price_history)
                .values((price_history::dsl::variation_id.eq(&var_id),price_history::dsl::price.eq(&price_insert),price_history::dsl::currency.eq(&currency_insert)))
                .execute(tx).await?;

            //Si el discount no está activo
            if !form.discount_active{
                //eliminamos el que tiene variation_id discount_type currency 
                diesel::delete(discounts::dsl::discounts)
                    .filter(discounts::dsl::variation_id.eq(&var_id))
                    .filter(discounts::dsl::discount_type.eq(0))
                    .filter(discounts::dsl::currency.eq(&currency_insert))
                    .execute(tx)
                    .await?;

                let alias_d_h = diesel::alias!(discount_history as alias_d_h);

                //actualizar la tabla de historial
                let id_query = alias_d_h
                    .select(alias_d_h.field(discount_history::dsl::id))
                    .filter(alias_d_h.field(discount_history::dsl::variation_id).eq(&var_id))
                    .filter(alias_d_h.field(discount_history::dsl::discount_type).eq(0))
                    .filter(alias_d_h.field(discount_history::dsl::currency).eq(&currency_insert))
                    .filter(alias_d_h.field(discount_history::dsl::end_date).is_null())
                    .order_by(alias_d_h.field(discount_history::dsl::created_at).desc())
                    .limit(1)
                    .for_update()
                    .skip_locked()
                    .single_value();

                update(discount_history::dsl::discount_history)
                    .filter(discount_history::dsl::id.nullable().eq(id_query))
                    .set(discount_history::dsl::end_date.eq(&now))
                    .execute(tx)
                    .await?;
                
            }else{
    
                //let result
                let new_discount = NewDiscount{
                    variation_id:var_id,
                    discount_type:&0,
                    percentage: None,
                    quantity:None,
                    discount_value:&form.discount,
                    currency:&currency_insert,
                    start_date:Some(&now),
                    end_date:None,
                };
                let new_discount_history = NewDiscountHistory{
                    variation_id:var_id,
                    discount_type:&0,
                    percentage: None,
                    quantity:None,
                    discount_value:&form.discount,
                    currency:&currency_insert,
                    start_date:Some(&now),
                    end_date:None,
                    created_at:&now
                };
    
    
                let result_discount = insert_into(discounts::dsl::discounts)
                    .values(new_discount)
                    .on_conflict(( discounts::dsl::variation_id,discounts::dsl::discount_type,discounts::dsl::currency))
                    .do_update()
                    .set((
                        discounts::dsl::percentage.eq(excluded(discounts::dsl::percentage)),
                        discounts::dsl::quantity.eq(excluded(discounts::dsl::quantity)),
                        discounts::dsl::discount_value.eq(excluded(discounts::dsl::discount_value)),
                        discounts::dsl::start_date.eq(excluded(discounts::dsl::start_date)),
                        discounts::dsl::end_date.eq(excluded(discounts::dsl::end_date)),
                        discounts::dsl::created_at.eq(excluded(discounts::dsl::created_at))
                    ))
                    .execute(tx).await?;
    
                let result_discount = insert_into(discount_history::dsl::discount_history)
                    .values(new_discount_history)
                    .execute(tx).await?;
            }
    
            Ok(1)
        })).await
}

pub async fn update_discount_variation(var_id:&Uuid,status_value:&i16,pool:&DbPool)-> Result<usize,Error>{
    use crate::schema::product_variations::dsl::*;

    let connection = &mut pool.get().await.unwrap();

    let result = update(product_variations)
        .filter(id.eq(var_id))
        .set(status.eq(status_value))
        .execute(connection)
        .await;
    result  
}



pub async fn get_discount_variation(var_id:&Uuid,disc_type:&i16,curr:&String,pool:&DbPool)->Result<Discounts,Error>{
    use crate::schema::discounts::dsl::*;

    let connection = &mut pool.get().await.unwrap();

    let result = discounts
        .filter(
            variation_id.eq(var_id).and(
            discount_type.eq(disc_type)).and(
            currency.eq(curr))
        )
        .first::<Discounts>(connection).await;
    result
}

pub async fn get_stock_variation(var_id:&Uuid,pool:&DbPool)->Result<i32,Error>{
    use crate::schema::product_variations::dsl::*;

    let connection = &mut pool.get().await.unwrap();

    let result = product_variations
        .filter(id.eq(var_id))
        .select(stock)
        .first::<i32>(connection)
        .await;
    //obtener solo el stock
    result
}

pub async fn set_stock_variation(var_id:&Uuid,new_stock:&i32,pool:&DbPool)->Result<usize,Error>{
    use crate::schema::product_variations::dsl::*;
    let connection = &mut pool.get().await.unwrap();

    let result = update(product_variations)
        .filter(id.eq(var_id))
        .set(stock.eq(new_stock))
        .execute(connection)
        .await;
    result
}

//Obtener si existe un usuario con correo registrado
pub async fn check_mail(mail:&String,pool: &DbPool)->Result<bool,Error>{
    use crate::schema::base_user::dsl::*;

    let connection = &mut pool.get().await.unwrap();

    let result = select(exists(
        base_user.filter(email.eq(mail))
    ))
    .get_result::<bool>(connection)
    .await;

    result
}
pub async fn check_username(user:&String,pool: &DbPool)->Result<bool,Error>{
    use crate::schema::base_user::dsl::*;

    let connection = &mut pool.get().await.unwrap();

    let result = select(exists(
        base_user.filter(username.eq(user))
    ))
    .get_result::<bool>(connection)
    .await;

    result
}
//obtener el usuario y en este caso, su contraseña hasheada
//Que devuelva un objeto con lo que se quiera  obtener del usuario
// Poner la restriccion de que el usuario esté activo
pub async fn get_user(username_in:&String,pool:&DbPool)->Result<UserAuth,Error>{
    use crate::schema::base_user::dsl::*;
    let connection = &mut pool.get().await.unwrap();

    let user = base_user
        .filter(username.eq(username_in))
        .select((
            id,
            username,
            name,
            lastname,
            email,
            password
        ))
        .first::<UserAuth>(connection)
        .await;

    user
}

pub async fn get_user_2(user_id:&Uuid,pool:&DbPool)->Result<UserAuth,Error>{
    use crate::schema::base_user::dsl::*;
    let connection = &mut pool.get().await.unwrap();

    let user = base_user
        .filter(id.eq(user_id))
        .select((
            id,
            username,
            name,
            lastname,
            email,
            password
        ))
        .first::<UserAuth>(connection)
        .await;

    user
}

//pasar modelo base de entrada? --> o un nuevo modelo?
//Se tiene que hashear la contraseña, además 
pub async fn new_user_base(username_in:&String,email_in:&String,password_in:&String,pool: &DbPool)->Result<Uuid,Error>{
    use crate::schema::base_user::dsl::*;
    use crate::models::models_x::NewBaseUserSimple;

    let connection = &mut pool.get().await.unwrap();

    let new_id = Uuid::new_v4();

    let user = NewBaseUserSimple{
        id:&new_id,
        username:&username_in,
        email:&email_in,
        password:password_in
    };

    let result = insert_into(base_user).values(user).execute(connection).await;

    match result{
        Ok(_)=>{
            return Ok(new_id)
        },
        Err(e)=>{
            return Err(e)
        }
    }
}

#[derive(Queryable, Debug, Clone)]
pub struct VariationPriceDetailed {
    pub variation_id: Uuid,
    pub price: BigDecimal,
}

pub async fn get_variations_prices_product(
    product_id: &Uuid, 
    pool: &DbPool
) -> Result<Vec<VariationPriceDetailed>, Error> {
    use crate::schema::product_variations::dsl as pv;
    use crate::schema::prices::dsl as p;
    
    let connection = &mut pool.get().await.unwrap();
    
    let results = pv::product_variations
        .inner_join(p::prices.on(pv::id.eq(p::variation_id)))
        .filter(pv::product_id.eq(product_id))
        .select((
            pv::id,
            p::price
        ))
        .order_by( p::price.asc())
        .load::<VariationPriceDetailed>(connection)
        .await?;
    
    Ok(results)
}


pub async fn insert_session_login<'a>(session_data: NewUserSession<'a>, pool:&DbPool) -> Result<usize, Error> {
    use crate::schema::user_sessions::dsl::*;
    let connection = &mut pool.get().await.unwrap();

    let result = insert_into(user_sessions)
        .values(session_data)
        .execute(connection).await;
    result
}
use crate::models::models_x::UserSession;
pub async fn check_session(jti_x:&String, pool:&DbPool)-> Result<UserSession,Error>{
    use crate::schema::user_sessions::dsl::*;
    let connection = &mut pool.get().await.unwrap();

    //let now = Utc::now().naive_utc();

    let result = user_sessions
        .filter(jti.eq(jti_x))
        // .filter(is_revoked.eq(false))
        // .filter(issued_at.lt(now))
        // .filter(expires_at.gt(now))
        .first::<UserSession>(connection).await;
    result
}

pub async fn get_product_cards1(pool:&DbPool)->Result<Vec<ProductCard1>,Error>{
    let connection = &mut pool.get().await.unwrap();
    let results = sql_query(r#"
        select p.id,
            p.name,
            p.brand,
            (SELECT img.image_url 
            FROM images_product img 
            WHERE img.product_id = p.id 
            ORDER BY img.is_main DESC, img.id ASC 
            LIMIT 1) as image_url,
            MIN(pr.price) price,
            pr.currency,
            s.store_name,
            s.id store_id
        from products p
        left join product_variations pv on p.id=pv.product_id
        left join prices pr on pv.id=pr.variation_id
        left join product_seller ps on p.id=ps.product_id
        left join seller s on ps.seller_id=s.id
        where p.status = 1
        group by p.id, p.name, p.brand, pr.currency, s.store_name, s.id
        order by p.name;
    "#)
        .load::<ProductCard1>(connection).await?;
    Ok(results)
}

pub async fn get_product_with_images1(user_id: &Uuid, product_id: &Uuid, pool:&DbPool)->Result<ProductWithImages1,Error>{
    let connection = &mut pool.get().await.unwrap();
    let results = sql_query(r#"
        select p.*,
            json_agg(json_build_object(
                'image_url', i.image_url,
                'is_main', i.is_main,
                'display_order', i.display_order
            )) FILTER (WHERE i.id IS NOT NULL) as images_product
        FROM products p
        LEFT JOIN images_product i on p.id = i.product_id
        LEFT JOIN product_seller ps on p.id = ps.product_id
        WHERE p.id = $1
            AND ps.seller_id = $2
        GROUP BY p.id
    "#).bind::<diesel::sql_types::Uuid,_>(product_id)
        .bind::<diesel::sql_types::Uuid,_>(user_id)
        .get_result::<ProductWithImages1>(connection).await?;
    Ok(results)
}

pub async fn get_product_for_page(product_id: &Uuid, pool:&DbPool)->Result<ProductForPage1,Error>{
    let connection = &mut pool.get().await.unwrap();
    let results = sql_query(r#"
        WITH variacion_prioritaria AS (
            SELECT v.id, v.attributes, pr.price, pr.currency
            FROM Product_variations v
            LEFT JOIN prices pr ON v.id = pr.variation_id
            WHERE v.product_id = $1 AND v.status = 1
            ORDER BY (v.stock > 0) DESC, v.stock DESC
            LIMIT 1
        ),
        stock_por_atributo AS (
            SELECT
                v.product_id,
                attr->>'name' as attr_name,
                attr->>'value' as attr_value,
                SUM(v.stock) as total_stock
            FROM Product_variations v,
            LATERAL jsonb_array_elements(v.attributes) AS attr
            WHERE v.product_id = $1 AND v.status = 1
            GROUP BY v.product_id, attr_name, attr_value
        ),
        todas_las_combinaciones AS (
            SELECT jsonb_agg(
                jsonb_build_object(
                    'attributes', v.attributes,
                    'price', pr.price,
                    'id', v.id,
                    'stock', v.stock
                )
                ORDER BY pr.price ASC NULLS LAST
            ) as mapa
            FROM Product_variations v
            LEFT JOIN prices pr ON v.id = pr.variation_id
            WHERE v.product_id = $1 AND v.status = 1
        )
        SELECT
            p.id,
            p.name,
            p.brand,
            p.description,
            pr.currency,
            s.store_name,
            s.id store_id,
            img_agg.lista_imagenes as images_product,
            (SELECT mapa FROM todas_las_combinaciones) as variant_map,
            (
                SELECT jsonb_agg(
                    jsonb_build_object(
                        'name', v_def->>'name',
                        'values', (
                            SELECT jsonb_agg(
                                jsonb_build_object(
                                    'value', val->>'value',
                                    'in_stock', COALESCE(sa.total_stock, 0) > 0,
                                    'is_default', EXISTS (
                                        SELECT 1 FROM variacion_prioritaria vp,
                                        LATERAL jsonb_array_elements(vp.attributes) as def_attr
                                        WHERE def_attr->>'name' = v_def->>'name'
                                        AND def_attr->>'value' = val->>'value'
                                    )
                                )
                            )
                            FROM jsonb_array_elements(v_def->'values') AS val
                            LEFT JOIN stock_por_atributo sa
                                ON sa.attr_name = v_def->>'name'
                                AND sa.attr_value = val->>'value'
                        )
                    )
                )
                FROM jsonb_array_elements(p.variations) AS v_def
            ) AS variations_with_stock_status
        FROM Products p
        LEFT JOIN LATERAL (
            SELECT jsonb_agg(jsonb_build_object(
                'image_url', img.image_url,
                'is_main', img.is_main,
                'display_order', img.display_order
            )) as lista_imagenes
            FROM images_product img
            WHERE img.product_id = p.id
        ) img_agg ON true
        LEFT JOIN Product_variations pv ON p.id = pv.product_id
        LEFT JOIN prices pr ON pv.id = pr.variation_id
        LEFT JOIN product_seller ps ON p.id = ps.product_id
        LEFT JOIN seller s ON ps.seller_id = s.id
        WHERE p.id = $1 AND p.status = 1
        GROUP BY p.id, p.name, p.brand, pr.currency, s.store_name, s.id, img_agg.lista_imagenes;
    "#).bind::<diesel::sql_types::Uuid,_>(product_id)
        .get_result::<ProductForPage1>(connection).await?;
    Ok(results)
}
