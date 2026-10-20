//! Imágenes de un producto en el panel (pestaña Imágenes de "Editar producto").

use diesel::result::Error;
use diesel::sql_query;
use diesel::sql_types::Uuid as SqlUuid;
use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

type DbPool = Pool<AsyncPgConnection>;

/// Deja `image_id` como la única principal del producto (la de la tarjeta del
/// listado, la primera de la ficha y la del carrito si la variación no tiene
/// propia). `Ok(false)` sin cambiar nada si la imagen no es de ese producto:
/// quien llama ya ha comprobado que el producto es del usuario (`product_access`).
pub async fn set_main_image(product_id: &Uuid, image_id: &Uuid, pool: &DbPool) -> Result<bool, Error> {
    let product_id = *product_id;
    let image_id = *image_id;
    let mut conn = pool.get().await.unwrap();

    conn.transaction::<bool, Error, _>(|conn| async move {
        let marked = sql_query("UPDATE images_product SET is_main = true WHERE id = $1 AND product_id = $2")
            .bind::<SqlUuid, _>(image_id)
            .bind::<SqlUuid, _>(product_id)
            .execute(conn)
            .await?;
        if marked == 0 {
            return Ok(false);
        }
        sql_query("UPDATE images_product SET is_main = false WHERE product_id = $1 AND id <> $2 AND is_main IS DISTINCT FROM false")
            .bind::<SqlUuid, _>(product_id)
            .bind::<SqlUuid, _>(image_id)
            .execute(conn)
            .await?;
        Ok(true)
    }.scope_boxed()).await
}
