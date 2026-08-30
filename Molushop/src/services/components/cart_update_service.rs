use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use uuid::Uuid;

use crate::schema::{carts, cart_products, product_variations};
use crate::models::models_x::{Cart, CartProduct, ProductVariation};
use crate::models::error::ServiceError;

type DbPool = Pool<AsyncPgConnection>;

/// Comprueba que la línea existe y que pertenece al carrito ACTIVO del usuario.
/// Es la única puerta de entrada a las mutaciones: sin esto cualquier usuario
/// autenticado podría tocar las líneas de otro pasando su UUID.
async fn get_owned_line(
    user_id: &Uuid,
    cart_product_id: &Uuid,
    conn: &mut AsyncPgConnection,
) -> Result<CartProduct, ServiceError> {
    let cp: CartProduct = cart_products::table
        .filter(cart_products::id.eq(cart_product_id))
        .first::<CartProduct>(conn)
        .await
        .map_err(|_| ServiceError::CartItemNotFound)?;

    let cart_id = cp.cart_id.ok_or(ServiceError::CartItemNotFound)?;

    let _cart: Cart = carts::table
        .filter(carts::id.eq(&cart_id))
        .filter(carts::user_id.eq(user_id))
        .filter(carts::status.eq(1_i16))
        .first::<Cart>(conn)
        .await
        .map_err(|_| ServiceError::CartItemNotFound)?;

    Ok(cp)
}

async fn touch_cart(cart_id: &Uuid, conn: &mut AsyncPgConnection) -> Result<(), ServiceError> {
    diesel::update(carts::table.find(cart_id))
        .set(carts::updated_at.eq(diesel::dsl::now))
        .execute(conn)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    Ok(())
}

/// Fija la cantidad de una línea. Una cantidad <= 0 elimina la línea,
/// que es lo que espera el usuario al bajar el contador hasta 0.
pub async fn update_cart_item(
    user_id: &Uuid,
    cart_product_id: &Uuid,
    quantity: i32,
    pool: &DbPool,
) -> Result<(), ServiceError> {
    if quantity <= 0 {
        return remove_cart_item(user_id, cart_product_id, pool).await;
    }

    let mut conn = pool.get().await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let cp = get_owned_line(user_id, cart_product_id, &mut conn).await?;
    let cart_id = cp.cart_id.ok_or(ServiceError::CartItemNotFound)?;
    let var_id = cp.product_var_id.ok_or(ServiceError::VariationNotFound)?;

    let variation: ProductVariation = product_variations::table
        .find(&var_id)
        .first(&mut conn)
        .await
        .map_err(|_| ServiceError::VariationNotFound)?;

    if variation.stock < quantity {
        return Err(ServiceError::NotEnoughStock);
    }

    diesel::update(cart_products::table.find(&cp.id))
        .set(cart_products::quantity.eq(quantity))
        .execute(&mut conn)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    touch_cart(&cart_id, &mut conn).await?;

    Ok(())
}

pub async fn remove_cart_item(
    user_id: &Uuid,
    cart_product_id: &Uuid,
    pool: &DbPool,
) -> Result<(), ServiceError> {
    let mut conn = pool.get().await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let cp = get_owned_line(user_id, cart_product_id, &mut conn).await?;
    let cart_id = cp.cart_id.ok_or(ServiceError::CartItemNotFound)?;

    diesel::delete(cart_products::table.find(&cp.id))
        .execute(&mut conn)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    touch_cart(&cart_id, &mut conn).await?;

    Ok(())
}
