//! La tienda vista desde la tienda pública (`/stores/{id}`): solo las ACTIVAS,
//! como en la portada, la búsqueda y la ficha. También el directorio `/stores`.

use chrono::{DateTime, Utc};
use diesel::result::Error;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Nullable, Text, Timestamptz, Uuid as SqlUuid};
use diesel::QueryableByName;
use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::RunQueryDsl;
use uuid::Uuid;
type DbPool = Pool<AsyncPgConnection>;

use crate::utils::db::get_connection;

#[derive(QueryableByName, Debug, Clone)]
pub struct PublicStore {
    #[diesel(sql_type = SqlUuid)]
    pub id: Uuid,
    #[diesel(sql_type = Text)]
    pub store_name: String,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: DateTime<Utc>,
    /// Productos a la venta (lo mismo que cuenta `product_service::get_product_cards`).
    #[diesel(sql_type = BigInt)]
    pub products: i64,
}

/// La tienda `store_id` si está ACTIVA (`seller.status = 1`); `None` si no
/// existe o está pendiente, inactiva o rechazada.
pub async fn get_active_store(store_id: &Uuid, pool: &DbPool) -> Result<Option<PublicStore>, Error> {
    let connection = &mut get_connection(pool).await?;
    let mut rows = sql_query(r#"
        select s.id, s.store_name::text as store_name, s.created_at,
            (select count(*) from product_seller ps
             join products p on p.id = ps.product_id and p.status = 1
             where ps.seller_id = s.id) as products
        from seller s
        where s.id = $1 and s.status = 1
    "#)
        .bind::<SqlUuid, _>(store_id)
        .load::<PublicStore>(connection).await?;
    Ok(rows.pop())
}

/// Una tienda en el directorio `/stores`.
#[derive(QueryableByName, Debug, Clone)]
pub struct StoreListItem {
    #[diesel(sql_type = SqlUuid)]
    pub id: Uuid,
    #[diesel(sql_type = Text)]
    pub store_name: String,
    /// Productos a la venta, contados como en `get_active_store`.
    #[diesel(sql_type = BigInt)]
    pub products: i64,
    /// La foto principal de su primer producto a la venta (por nombre) que tenga
    /// alguna; `None` si ninguno tiene foto (no hay logo de tienda).
    #[diesel(sql_type = Nullable<Text>)]
    pub image_url: Option<String>,
}

/// Las tiendas ACTIVAS, por nombre sin contar mayúsculas ni acentos. También
/// las que aún no tienen productos: están abiertas. Sin paginar: son pocas;
/// si un día son muchas, se pagina como la búsqueda.
pub async fn list_active_stores(pool: &DbPool) -> Result<Vec<StoreListItem>, Error> {
    let connection = &mut get_connection(pool).await?;
    sql_query(r#"
        select s.id, s.store_name::text as store_name,
            (select count(*) from product_seller ps
             join products p on p.id = ps.product_id and p.status = 1
             where ps.seller_id = s.id) as products,
            (select img.image_url
             from product_seller ps
             join products p on p.id = ps.product_id and p.status = 1
             join lateral (
                 select i.image_url from images_product i
                 where i.product_id = p.id
                 order by i.is_main desc nulls last, i.display_order asc, i.id asc
                 limit 1
             ) img on true
             where ps.seller_id = s.id
             order by p.name asc, p.id asc
             limit 1) as image_url
        from seller s
        where s.status = 1
        order by f_unaccent(lower(s.store_name)) asc, s.id asc
    "#)
        .load::<StoreListItem>(connection).await
}
