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

#[derive(diesel::QueryableByName)]
struct DeletedRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    image_url: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Bool>)]
    is_main: Option<bool>,
}

#[derive(diesel::QueryableByName)]
struct IdRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
}

#[derive(diesel::QueryableByName)]
struct InUse {
    #[diesel(sql_type = diesel::sql_types::Bool)]
    in_use: bool,
}

/// Lo que ha pasado al borrar una imagen.
#[derive(Debug)]
pub struct DeletedImage {
    pub image_url: String,
    /// Si era la principal, la que pasa a serlo (la primera por `display_order`).
    pub new_main: Option<Uuid>,
    /// Ninguna otra imagen ni ningún pedido usa la URL: se puede borrar el
    /// fichero de S3. Los pedidos guardan su propia copia de la URL
    /// (`order_items.image_url`) y sin el fichero se quedarían sin foto.
    pub file_unused: bool,
}

/// Borra la imagen del producto (sus enlaces con variaciones se van con ella,
/// ON DELETE CASCADE) y, si era la principal, hace principal a la siguiente.
/// No toca S3: lo hace quien llama, después, si `file_unused`. `Ok(None)` sin
/// cambiar nada si la imagen no es de ese producto: quien llama ya ha
/// comprobado que el producto es del usuario (`product_access`).
pub async fn delete_image(product_id: &Uuid, image_id: &Uuid, pool: &DbPool) -> Result<Option<DeletedImage>, Error> {
    let product_id = *product_id;
    let image_id = *image_id;
    let mut conn = pool.get().await.unwrap();

    conn.transaction::<Option<DeletedImage>, Error, _>(|conn| async move {
        let deleted: Option<DeletedRow> = sql_query("DELETE FROM images_product WHERE id = $1 AND product_id = $2 RETURNING image_url, is_main")
            .bind::<SqlUuid, _>(image_id)
            .bind::<SqlUuid, _>(product_id)
            .get_results(conn)
            .await?
            .pop();
        let Some(deleted) = deleted else {
            return Ok(None);
        };

        let mut new_main = None;
        if deleted.is_main == Some(true) {
            let next: Option<IdRow> = sql_query("SELECT id FROM images_product WHERE product_id = $1 ORDER BY display_order, id LIMIT 1")
                .bind::<SqlUuid, _>(product_id)
                .get_results(conn)
                .await?
                .pop();
            if let Some(next) = next {
                sql_query("UPDATE images_product SET is_main = true WHERE id = $1")
                    .bind::<SqlUuid, _>(next.id)
                    .execute(conn)
                    .await?;
                new_main = Some(next.id);
            }
        }

        let in_use: InUse = sql_query("SELECT EXISTS (SELECT 1 FROM images_product WHERE image_url = $1)
                OR EXISTS (SELECT 1 FROM order_items WHERE image_url = $1) AS in_use")
            .bind::<diesel::sql_types::Text, _>(&deleted.image_url)
            .get_result(conn)
            .await?;

        Ok(Some(DeletedImage { image_url: deleted.image_url, new_main, file_unused: !in_use.in_use }))
    }.scope_boxed()).await
}
