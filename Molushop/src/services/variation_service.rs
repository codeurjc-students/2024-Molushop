//! Variaciones de un producto: alta, lectura, stock e identificadores (EAN, ISBN...).

use diesel::prelude::*;
use diesel::result::Error;
use diesel::upsert::excluded;
use diesel::{insert_into, update};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use serde_json::Value;
use uuid::Uuid;

use crate::models::components::edit_product_variation_model::Identifier;
use crate::models::models_x::{IdentifierVariation, NewIdentifierVariation, NewProductVariation1, Price, ProductVariation};
use crate::models::product_variation::VariationValue;
use crate::utils::db::get_connection;


type DbPool = Pool<AsyncPgConnection>;

 pub async fn insert_product_variations(prod_id:&Uuid,combinations: Vec<Vec<VariationValue>>,connection:&mut AsyncPgConnection) -> Result<Vec<Uuid>,Error>{
    use crate::schema::product_variations::dsl::*;
    let mut new_ids:Vec<Uuid>= Vec::new();
    for combination in combinations{
        let new_id = Uuid::new_v4();
        let json_attributes:Value = serde_json::to_value(&combination).unwrap();
        let new_variation = NewProductVariation1{
            id: &new_id,
            product_id: prod_id,
            attributes: Some(&json_attributes),
        };
        new_ids.push(new_id);
        insert_into(product_variations).values(new_variation).execute(connection).await?;
    }
    Ok(new_ids)
 }

 pub async fn insert_product_variation(prod_id:&Uuid, connection:&mut AsyncPgConnection) -> Result<Uuid,Error>{
    use crate::schema::product_variations::dsl::*;
    let new_id = Uuid::new_v4();
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

pub async fn get_product_variations(id_product:&Uuid,pool:&DbPool) -> Result<Vec<ProductVariation>,Error> {
    use crate::schema::product_variations::dsl::*;
    //let connection = &mut establish_connection();
    let connection = &mut get_connection(pool).await?;
    let results = product_variations
        .filter(product_id.eq(id_product))
        .load::<ProductVariation>(connection).await;
    results
}

pub async fn get_product_variation(id_variation:&Uuid,pool:&DbPool) -> Result<ProductVariation,Error> {
    use crate::schema::product_variations::dsl::*;
    //let connection = &mut establish_connection();
    let connection = &mut get_connection(pool).await?;
    let result = product_variations.filter(id.eq(id_variation)).first::<ProductVariation>(connection).await;
    result
}

pub async fn get_variations_with_prices(prod_id:&Uuid,curr:&String,pool: &DbPool) -> Result<Vec<(ProductVariation, Price)>, Error> {
    use crate::schema::product_variations::dsl as pv;
    use crate::schema::prices::dsl as p;
    
    let connection = &mut get_connection(pool).await?;
    
    let results = pv::product_variations
        .inner_join(p::prices.on(pv::id.eq(p::variation_id)))
        .filter(p::currency.eq(curr))
        .filter(pv::product_id.eq(prod_id))
        .load::<(ProductVariation, Price)>(connection)
        .await?;
    
    Ok(results)
}

pub async fn get_variation_identifiers(var_id:&Uuid, pool:&DbPool) -> Result<Vec<IdentifierVariation>,Error>{
    use crate::schema::product_variations_identifiers::dsl::*;
    let connection = &mut get_connection(pool).await?;
    let result = product_variations_identifiers
        .filter(product_variation_id.eq(var_id))
        .load::<IdentifierVariation>(connection).await;
    result
}

////NUEVO///
pub async fn set_variation_identifiers(
    vec_identifiers: &[Identifier],
    var_id: &Uuid,
    pool: &DbPool
) -> Result<usize, Error> {
    use crate::schema::product_variations_identifiers::dsl::*;
    let connection = &mut get_connection(pool).await?;
    
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

pub async fn get_identifier_options(pool: &DbPool) -> Result<Vec<String>, Error> {
    use crate::schema::identifiers_var::dsl::*;
    
    let connection = &mut get_connection(pool).await?;
    let results = identifiers_var
        .select(value)
        .order_by(value.asc())
        .load::<String>(connection)
        .await;
    
    results
}

pub async fn get_variation_stock(var_id:&Uuid,pool:&DbPool)->Result<i32,Error>{
    use crate::schema::product_variations::dsl::*;

    let connection = &mut get_connection(pool).await?;

    let result = product_variations
        .filter(id.eq(var_id))
        .select(stock)
        .first::<i32>(connection)
        .await;
    //obtener solo el stock
    result
}

pub async fn set_variation_stock(var_id:&Uuid,new_stock:&i32,pool:&DbPool)->Result<usize,Error>{
    use crate::schema::product_variations::dsl::*;
    let connection = &mut get_connection(pool).await?;

    let result = update(product_variations)
        .filter(id.eq(var_id))
        .set(stock.eq(new_stock))
        .execute(connection)
        .await;
    result
}
