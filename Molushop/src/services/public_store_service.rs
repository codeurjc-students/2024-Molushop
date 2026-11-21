//! La tienda vista desde la tienda pública (`/stores/{id}`): solo las ACTIVAS,
//! como en la portada, la búsqueda y la ficha.

use chrono::{DateTime, Utc};
use diesel::result::Error;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Text, Timestamptz, Uuid as SqlUuid};
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
