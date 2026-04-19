use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use bigdecimal::BigDecimal;
use uuid::Uuid;

use crate::schema::{carts, cart_products, product_variations, prices};
use crate::models::models_x::{NewCart, NewCartProduct, Cart, CartProduct, ProductVariation, Price};
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

    // 3. Buscar o crear carrito activo (status = 0) -- NOTA: en la migración status default es 1
    // Usamos status = 1 (ACTIVE) según la migración: 0=DRAFT, 1=ACTIVE, 2=INACTIVE
    let cart: Cart = match carts::table
        .filter(carts::user_id.eq(user_id))
        .filter(carts::status.eq(1_i16))
        .first::<Cart>(&mut conn)
        .await
    {
        Ok(c) => c,
        Err(diesel::result::Error::NotFound) => {
            let new_id = Uuid::new_v4();
            let new_cart = NewCart {
                id: &new_id,
                user_id,
                status: 1,
            };
            diesel::insert_into(carts::table)
                .values(&new_cart)
                .get_result::<Cart>(&mut conn)
                .await
                .map_err(|e| ServiceError::InternalServerError(e.to_string()))?
        }
        Err(e) => return Err(ServiceError::InternalServerError(e.to_string())),
    };

    // 4. Verificar si ya existe este producto en el carrito
    let existing: Option<CartProduct> = cart_products::table
        .filter(cart_products::cart_id.eq(&cart.id))
        .filter(cart_products::product_var_id.eq(product_var_id))
        .first::<CartProduct>(&mut conn)
        .await
        .optional()
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    match existing {
        Some(cp) => {
            // Actualizar cantidad y precio
            let new_qty = cp.quantity + quantity;
            if variation.stock < new_qty {
                return Err(ServiceError::NotEnoughStock);
            }
            diesel::update(cart_products::table.find(&cp.id))
                .set((
                    cart_products::quantity.eq(new_qty),
                    cart_products::price_at_time_of_addition.eq(Some(&current_price.price)),
                ))
                .execute(&mut conn)
                .await
                .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
        }
        None => {
            // Insertar nueva línea
            let new_cp = NewCartProduct {
                id: &Uuid::new_v4(),
                cart_id: &cart.id,
                product_var_id,
                quantity,
                price_at_time_of_addition: Some(&current_price.price),
            };
            diesel::insert_into(cart_products::table)
                .values(&new_cp)
                .execute(&mut conn)
                .await
                .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
        }
    }

    // 5. Actualizar updated_at del carrito
    diesel::update(carts::table.find(&cart.id))
        .set(carts::updated_at.eq(diesel::dsl::now))
        .execute(&mut conn)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    Ok("Producto añadido al carrito".to_string())
}
