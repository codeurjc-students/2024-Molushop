//! En qué variaciones sale cada imagen de un producto (pestaña Imágenes de
//! "Editar producto"). Se guarda en `images_product_variations`: una imagen sin
//! enlaces es del producto entero y sale en todas.

use diesel::result::Error;
use diesel::sql_query;
use diesel::sql_types::{Array, BigInt, Uuid as SqlUuid};
use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

type DbPool = Pool<AsyncPgConnection>;

#[derive(diesel::QueryableByName)]
struct Count {
    #[diesel(sql_type = BigInt)]
    n: i64,
}

/// Sustituye los enlaces de la imagen por `variation_ids` (vacío: sale en
/// todas), en una transacción. `Ok(false)` sin cambiar nada si la imagen o
/// alguna variación no son de ese producto: quien llama ya ha comprobado que
/// el producto es del usuario (`product_access`).
pub async fn set_image_variations(
    product_id: &Uuid,
    image_id: &Uuid,
    variation_ids: &[Uuid],
    pool: &DbPool,
) -> Result<bool, Error> {
    let mut ids = variation_ids.to_vec();
    ids.sort();
    ids.dedup();
    let product_id = *product_id;
    let image_id = *image_id;
    let mut conn = pool.get().await.unwrap();

    conn.transaction::<bool, Error, _>(|conn| async move {
        let image = sql_query("SELECT count(*) AS n FROM images_product WHERE id = $1 AND product_id = $2")
            .bind::<SqlUuid, _>(image_id)
            .bind::<SqlUuid, _>(product_id)
            .get_result::<Count>(conn)
            .await?;
        let owned = sql_query(
            "SELECT count(*) AS n FROM product_variations WHERE product_id = $1 AND id = ANY($2)"
        )
            .bind::<SqlUuid, _>(product_id)
            .bind::<Array<SqlUuid>, _>(&ids)
            .get_result::<Count>(conn)
            .await?;
        if image.n != 1 || owned.n != ids.len() as i64 {
            return Ok(false);
        }

        sql_query("DELETE FROM images_product_variations WHERE image_id = $1")
            .bind::<SqlUuid, _>(image_id)
            .execute(conn).await?;
        sql_query(
            "INSERT INTO images_product_variations (image_id, variation_id)
             SELECT $1, v FROM unnest($2::uuid[]) AS v"
        )
            .bind::<SqlUuid, _>(image_id)
            .bind::<Array<SqlUuid>, _>(&ids)
            .execute(conn).await?;
        Ok(true)
    }.scope_boxed()).await
}
