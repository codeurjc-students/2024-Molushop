use std::collections::HashSet;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Pool;
use uuid::Uuid;

use crate::constants::product_status::PRODUCT_ACTIVE;
use crate::models::error::ServiceError;
use crate::schema::{favorites, products};

type DbPool = Pool<AsyncPgConnection>;

/// Guarda un producto en los favoritos del usuario.
///
/// Solo productos activos: el corazón solo sale en la tienda, pero el POST se
/// puede hacer a mano con el id de un borrador o de uno borrado. Volver a
/// guardar uno que ya está (doble clic, otra pestaña) no es un error: el
/// `uq_favorite_user_product` de la tabla lo deja en DO NOTHING.
pub async fn add_favorite(
    user_id: &Uuid,
    product_id: &Uuid,
    pool: &DbPool,
) -> Result<(), ServiceError> {
    let connection = &mut pool
        .get()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let active: bool = diesel::select(diesel::dsl::exists(
        products::table
            .filter(products::id.eq(product_id))
            .filter(products::status.eq(PRODUCT_ACTIVE)),
    ))
    .get_result(connection)
    .await?;
    if !active {
        return Err(ServiceError::ProductNotFound);
    }

    diesel::insert_into(favorites::table)
        .values((
            favorites::id.eq(Uuid::new_v4()),
            favorites::user_id.eq(user_id),
            favorites::product_id.eq(product_id),
        ))
        .on_conflict((favorites::user_id, favorites::product_id))
        .do_nothing()
        .execute(connection)
        .await?;

    Ok(())
}

/// Quita un producto de los favoritos del usuario. Sin estado que comprobar:
/// se puede quitar aunque el producto ya no esté activo (la página de
/// favoritos los enseña como "No disponible" para eso).
///
/// Como en `delete_review`, el `user_id` va en el WHERE y no quitar nada (ya no
/// estaba) no es un error.
pub async fn remove_favorite(
    user_id: &Uuid,
    product_id: &Uuid,
    pool: &DbPool,
) -> Result<(), ServiceError> {
    let connection = &mut pool
        .get()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    diesel::delete(
        favorites::table
            .filter(favorites::user_id.eq(user_id))
            .filter(favorites::product_id.eq(product_id)),
    )
    .execute(connection)
    .await?;

    Ok(())
}

/// Cuáles de estos productos tiene el usuario en favoritos, para pintar el
/// corazón lleno o vacío. Una sola consulta para toda una rejilla de tarjetas,
/// no una por tarjeta.
pub async fn favorite_ids_among(
    user_id: &Uuid,
    product_ids: &[Uuid],
    pool: &DbPool,
) -> Result<HashSet<Uuid>, ServiceError> {
    if product_ids.is_empty() {
        return Ok(HashSet::new());
    }

    let connection = &mut pool
        .get()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let ids: Vec<Uuid> = favorites::table
        .filter(favorites::user_id.eq(user_id))
        .filter(favorites::product_id.eq_any(product_ids))
        .select(favorites::product_id)
        .load(connection)
        .await?;

    Ok(ids.into_iter().collect())
}

/// Los favoritos del usuario, del último guardado al primero (la página de
/// favoritos). Incluye los que ya no están activos.
pub async fn favorite_product_ids(
    user_id: &Uuid,
    pool: &DbPool,
) -> Result<Vec<Uuid>, ServiceError> {
    let connection = &mut pool
        .get()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let ids = favorites::table
        .filter(favorites::user_id.eq(user_id))
        .order(favorites::added_at.desc())
        .select(favorites::product_id)
        .load(connection)
        .await?;

    Ok(ids)
}
