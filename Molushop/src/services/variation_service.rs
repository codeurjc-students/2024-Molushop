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
use crate::models::db::{IdentifierVariation, NewIdentifierVariation, NewProductVariation, Price, ProductVariation};
use crate::models::product_variation::VariationValue;
use crate::utils::db::get_connection;


type DbPool = Pool<AsyncPgConnection>;

 pub async fn insert_product_variations(prod_id:&Uuid,combinations: Vec<Vec<VariationValue>>,connection:&mut AsyncPgConnection) -> Result<Vec<Uuid>,Error>{
    use crate::schema::product_variations::dsl::*;
    let mut new_ids:Vec<Uuid>= Vec::new();
    for combination in combinations{
        let new_id = Uuid::new_v4();
        let json_attributes:Value = serde_json::to_value(&combination).unwrap();
        let new_variation = NewProductVariation{
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
    let new_variation = NewProductVariation{
        id: &new_id,
        product_id: prod_id,
        attributes: None,
    };
    insert_into(product_variations).values(new_variation).execute(connection).await?;
    Ok(new_id)
 }

pub async fn get_product_variations(product_id: &Uuid, pool: &DbPool) -> Result<Vec<ProductVariation>, Error> {
    use crate::schema::product_variations as pv;
    let connection = &mut get_connection(pool).await?;
    pv::table.filter(pv::product_id.eq(product_id)).load::<ProductVariation>(connection).await
}

pub async fn get_product_variation(variation_id: &Uuid, pool: &DbPool) -> Result<ProductVariation, Error> {
    use crate::schema::product_variations as pv;
    let connection = &mut get_connection(pool).await?;
    pv::table.find(variation_id).first::<ProductVariation>(connection).await
}

/// Las variaciones del producto con su precio en esa moneda (las que no
/// tienen precio no salen).
pub async fn get_variations_with_prices(product_id: &Uuid, currency: &str, pool: &DbPool) -> Result<Vec<(ProductVariation, Price)>, Error> {
    use crate::schema::product_variations as pv;
    use crate::schema::prices as p;
    let connection = &mut get_connection(pool).await?;
    pv::table
        .inner_join(p::table.on(pv::id.eq(p::variation_id)))
        .filter(p::currency.eq(currency))
        .filter(pv::product_id.eq(product_id))
        .load::<(ProductVariation, Price)>(connection)
        .await
}

pub async fn get_variation_identifiers(variation_id: &Uuid, pool: &DbPool) -> Result<Vec<IdentifierVariation>, Error> {
    use crate::schema::product_variations_identifiers as pvi;
    let connection = &mut get_connection(pool).await?;
    pvi::table
        .filter(pvi::product_variation_id.eq(variation_id))
        .load::<IdentifierVariation>(connection)
        .await
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
            
            insert_into(product_variations_identifiers)
                .values(&insert_identifiers)
                .on_conflict((product_variation_id, identifier))
                .do_update()
                .set(value.eq(excluded(value)))
                .execute(tx)
                .await
        }))
        .await
}

/// Los nombres de identificador que se ofrecen al elegir (EAN, ISBN...).
pub async fn get_identifier_options(pool: &DbPool) -> Result<Vec<String>, Error> {
    use crate::schema::identifiers_var;
    let connection = &mut get_connection(pool).await?;
    identifiers_var::table
        .select(identifiers_var::value)
        .order_by(identifiers_var::value.asc())
        .load::<String>(connection)
        .await
}

pub async fn get_variation_stock(variation_id: &Uuid, pool: &DbPool) -> Result<i32, Error> {
    use crate::schema::product_variations as pv;
    let connection = &mut get_connection(pool).await?;
    pv::table.find(variation_id).select(pv::stock).first::<i32>(connection).await
}

pub async fn set_variation_stock(variation_id: &Uuid, stock: i32, pool: &DbPool) -> Result<usize, Error> {
    use crate::schema::product_variations as pv;
    let connection = &mut get_connection(pool).await?;
    update(pv::table.find(variation_id))
        .set(pv::stock.eq(stock))
        .execute(connection)
        .await
}
