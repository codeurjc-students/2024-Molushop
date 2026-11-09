//! Precios y descuentos de las variaciones.

use bigdecimal::BigDecimal;
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::upsert::excluded;
use diesel::{insert_into, update};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::models::components::edit_product_variation_model::PriceData;
use crate::models::db::{Discounts, NewDiscount, NewDiscountHistory, NewPrice, Price};
use crate::utils::db::get_connection;

type DbPool = Pool<AsyncPgConnection>;

pub async fn insert_variation_prices(
    variations_ids: &[Uuid],
    price_insert: &BigDecimal,
    currency_insert: &str,
    connection: &mut AsyncPgConnection
) -> Result<usize, Error> {
    use crate::schema::prices::dsl::*;
    //crear los modelos de insercion de precio
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

/// El precio de la variación en esa moneda, sin descuento (va aparte:
/// `get_variation_discount`).
pub async fn get_variation_price(variation_id: &Uuid, currency: &str, pool: &DbPool) -> Result<Price, Error> {
    let connection = &mut get_connection(pool).await?;
    crate::schema::prices::table
        .filter(crate::schema::prices::variation_id.eq(variation_id))
        .filter(crate::schema::prices::currency.eq(currency))
        .first::<Price>(connection)
        .await
}

pub async fn update_variation_price(form:&PriceData, var_id:&Uuid,price_insert: &BigDecimal,currency_insert: &str,pool:&DbPool)-> Result<usize,Error>{
    use crate::schema::prices;
     //::dsl::nombre
    
    use crate::schema::discounts;
    use crate::schema::discount_history;

    let connection = &mut get_connection(pool).await?;

    connection.build_transaction()
        .run(async move |tx| {
            //Primero actualizar el precio 
            let now = chrono::Utc::now().naive_utc();

            // INSERT ... ON CONFLICT y no un UPDATE a secas: una variación sin
            // fila en `prices` no se actualizaba (0 filas, sin error) y no había
            // forma de ponerle precio desde el panel.
            //
            // `price_history` NO se escribe aquí: lo hace el trigger
            // `price_history_trigger` (AFTER INSERT OR UPDATE ON prices), que
            // además cierra la fila anterior. Insertarlo también a mano dejaba
            // dos filas por cada cambio.
            insert_into(prices::dsl::prices)
                .values((
                    prices::dsl::variation_id.eq(&var_id),
                    prices::dsl::price.eq(&price_insert),
                    prices::dsl::currency.eq(currency_insert),
                    prices::dsl::start_date.eq(&now)
                ))
                .on_conflict((prices::dsl::variation_id, prices::dsl::currency))
                .do_update()
                .set((
                    prices::dsl::price.eq(&price_insert),
                    prices::dsl::start_date.eq(&now)
                ))
                .execute(tx).await?;

            //Si el discount no está activo
            if !form.discount_active{
                //eliminamos el que tiene variation_id discount_type currency 
                diesel::delete(discounts::dsl::discounts)
                    .filter(discounts::dsl::variation_id.eq(&var_id))
                    .filter(discounts::dsl::discount_type.eq(0))
                    .filter(discounts::dsl::currency.eq(currency_insert))
                    .execute(tx)
                    .await?;

                let alias_d_h = diesel::alias!(discount_history as alias_d_h);

                //actualizar la tabla de historial
                let id_query = alias_d_h
                    .select(alias_d_h.field(discount_history::dsl::id))
                    .filter(alias_d_h.field(discount_history::dsl::variation_id).eq(&var_id))
                    .filter(alias_d_h.field(discount_history::dsl::discount_type).eq(0))
                    .filter(alias_d_h.field(discount_history::dsl::currency).eq(currency_insert))
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
    
                let new_discount = NewDiscount{
                    variation_id:var_id,
                    discount_type:&0,
                    percentage: None,
                    quantity:None,
                    discount_value:&form.discount,
                    currency:currency_insert,
                    start_date:Some(&now),
                    end_date:None,
                };
                let new_discount_history = NewDiscountHistory{
                    variation_id:var_id,
                    discount_type:&0,
                    percentage: None,
                    quantity:None,
                    discount_value:&form.discount,
                    currency:currency_insert,
                    start_date:Some(&now),
                    end_date:None,
                    created_at:&now
                };

                let _result_discount = insert_into(discounts::dsl::discounts)
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
    
                let _result_discount = insert_into(discount_history::dsl::discount_history)
                    .values(new_discount_history)
                    .execute(tx).await?;
            }
    
            Ok(1)
        }).await
}

pub async fn get_variation_discount(variation_id: &Uuid, discount_type: i16, currency: &str, pool: &DbPool) -> Result<Discounts, Error> {
    use crate::schema::discounts;
    let connection = &mut get_connection(pool).await?;
    discounts::table
        .filter(discounts::variation_id.eq(variation_id))
        .filter(discounts::discount_type.eq(discount_type))
        .filter(discounts::currency.eq(currency))
        .first::<Discounts>(connection)
        .await
}

#[derive(Queryable, Debug, Clone)]
pub struct VariationPriceDetailed {
    pub variation_id: Uuid,
    pub price: BigDecimal,
}

pub async fn get_product_variation_prices(
    product_id: &Uuid, 
    pool: &DbPool
) -> Result<Vec<VariationPriceDetailed>, Error> {
    use crate::schema::product_variations::dsl as pv;
    use crate::schema::prices::dsl as p;
    
    let connection = &mut get_connection(pool).await?;
    
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
