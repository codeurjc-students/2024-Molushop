use diesel::prelude::*;
use diesel::sql_query;
use diesel_async::RunQueryDsl;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use bigdecimal::BigDecimal;
use uuid::Uuid;

use crate::schema::{carts, cart_products, product_variations, prices};
use diesel::OptionalExtension;
use crate::models::models_x::{Cart, ProductVariation, Price};
use crate::models::error::ServiceError;

type DbPool = Pool<AsyncPgConnection>;

pub async fn add_to_cart(
    user_id: &Uuid,
    product_var_id: &Uuid,
    quantity: i32,
    pool: &DbPool,
) -> Result<String, ServiceError> {
    let mut conn = pool.get().await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    // 1. Verificar que la variación existe y tiene stock
    let variation: ProductVariation = product_variations::table
        .find(product_var_id)
        .first(&mut conn)
        .await
        .map_err(|_| ServiceError::VariationNotFound)?;

    if variation.stock < quantity {
        return Err(ServiceError::NotEnoughStock);
    }

    // 2. Obtener el precio actual
    let current_price: Price = prices::table
        .filter(prices::variation_id.eq(product_var_id))
        .order(prices::start_date.desc())
        .first(&mut conn)
        .await
        .map_err(|e| ServiceError::InternalServerError(format!("No price found: {}", e)))?;

    // 3. Buscar o crear el carrito ACTIVO (status = 1; 0=DRAFT, 2=INACTIVE).
    // El INSERT ... ON CONFLICT DO NOTHING se apoya en el índice parcial
    // uq_active_cart: si dos peticiones simultáneas intentan crear el carrito,
    // una gana y la otra no hace nada; el SELECT posterior devuelve el mismo
    // carrito a las dos.
    sql_query(r#"
        INSERT INTO carts (id, user_id, status)
        VALUES ($1, $2, 1)
        ON CONFLICT DO NOTHING
    "#)
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .execute(&mut conn)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let cart: Cart = carts::table
        .filter(carts::user_id.eq(user_id))
        .filter(carts::status.eq(1_i16))
        .first::<Cart>(&mut conn)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    // 4. Insertar la línea o sumar a la existente, en una sola sentencia.
    // La constraint uq_cart_product hace imposible duplicar la línea, y el
    // WHERE del DO UPDATE deja la comprobación de stock dentro de la misma
    // operación atómica: si el total resultante no cabe en el stock no se
    // actualiza ninguna fila y `affected` vale 0.
    let affected = sql_query(r#"
        INSERT INTO cart_products (id, cart_id, product_var_id, quantity, price_at_time_of_addition)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (cart_id, product_var_id) DO UPDATE
            SET quantity = cart_products.quantity + EXCLUDED.quantity,
                price_at_time_of_addition = EXCLUDED.price_at_time_of_addition
            WHERE cart_products.quantity + EXCLUDED.quantity <= $6
    "#)
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Uuid, _>(&cart.id)
        .bind::<diesel::sql_types::Uuid, _>(product_var_id)
        .bind::<diesel::sql_types::Integer, _>(quantity)
        .bind::<diesel::sql_types::Numeric, _>(&current_price.price)
        .bind::<diesel::sql_types::Integer, _>(variation.stock)
        .execute(&mut conn)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    if affected == 0 {
        return Err(ServiceError::NotEnoughStock);
    }

    // 5. Actualizar updated_at del carrito
    diesel::update(carts::table.find(&cart.id))
        .set(carts::updated_at.eq(diesel::dsl::now))
        .execute(&mut conn)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    Ok("Producto añadido al carrito".to_string())
}

pub async fn get_cart_item_count(
    user_id: &Uuid,
    pool: &DbPool,
) -> i64 {
    let mut conn = match pool.get().await {
        Ok(c) => c,
        Err(_) => return 0,
    };

    let cart: Option<Cart> = carts::table
        .filter(carts::user_id.eq(user_id))
        .filter(carts::status.eq(1_i16))
        .first::<Cart>(&mut conn)
        .await
        .optional()
        .unwrap_or(None);

    match cart {
        Some(c) => {
            cart_products::table
                .filter(cart_products::cart_id.eq(&c.id))
                .select(diesel::dsl::sum(cart_products::quantity))
                .first::<Option<i64>>(&mut conn)
                .await
                .unwrap_or(None)
                .unwrap_or(0)
        }
        None => 0,
    }
}
