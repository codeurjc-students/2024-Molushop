use std::collections::HashSet;

use diesel::prelude::*;
use diesel::sql_query;
use diesel_async::RunQueryDsl;
use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Pool;
use uuid::Uuid;

use crate::constants::product_status::PRODUCT_ACTIVE;
use crate::models::components::product_card_model::ProductCardData;
use crate::models::error::ServiceError;
use crate::models::db::ProductCardRow;
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

/// Una tarjeta de "Mis favoritos": la de siempre más si aún se vende.
#[derive(QueryableByName)]
struct FavoriteCardRow {
    #[diesel(embed)]
    card: ProductCardRow,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    available: bool,
}

/// Las tarjetas de "Mis favoritos", del último guardado al primero, con el
/// corazón lleno. A diferencia de las de la portada (`get_product_cards`),
/// salen también las que ya no se venden, marcadas: si desaparecieran sin más,
/// el usuario no sabría qué ha pasado con lo que guardó.
pub async fn favorite_cards(user_id: &Uuid, pool: &DbPool) -> Result<Vec<ProductCardData>, ServiceError> {
    let connection = &mut pool
        .get()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let rows: Vec<FavoriteCardRow> = sql_query(r#"
        select p.id,
            p.name,
            p.brand,
            (select img.image_url
            from images_product img
            where img.product_id = p.id
            order by img.is_main desc, img.id asc
            limit 1) as image_url,
            pr.price,
            pr.currency,
            s.store_name,
            s.id store_id,
            (select coalesce(avg(r.rating_value), 0)::float8 from ratings r where r.product_id = p.id) as rating_average,
            (select count(*) from ratings r where r.product_id = p.id) as rating_count,
            -- Lo mismo que exige la portada: producto activo de una tienda activa.
            (p.status = 1 and s.id is not null) as available
        from favorites f
        join products p on p.id = f.product_id
        -- El precio "desde", como en get_product_cards.
        left join lateral (
            select pr2.price, pr2.currency
            from product_variations pv
            join prices pr2 on pr2.variation_id = pv.id
            where pv.product_id = p.id and pv.status = 1
            order by pr2.price asc
            limit 1
        ) pr on true
        -- LEFT y no JOIN: sin tienda activa la tarjeta sale igual, como no disponible.
        left join lateral (
            select s2.id, s2.store_name
            from product_seller ps
            join seller s2 on s2.id = ps.seller_id and s2.status = 1
            where ps.product_id = p.id
            limit 1
        ) s on true
        where f.user_id = $1
        order by f.added_at desc, p.id;
    "#)
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .load(connection)
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| ProductCardData {
            is_favorite: true,
            available: row.available,
            ..ProductCardData::from(row.card)
        })
        .collect())
}
