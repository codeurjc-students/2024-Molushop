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

use crate::services::search_service::name_matches;
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
            -- Un join y no un lateral por producto: así solo mira los que tienen
            -- foto (con 5000 productos demo, 32 -> 11 ms la consulta entera).
            (select i.image_url
             from product_seller ps
             join products p on p.id = ps.product_id and p.status = 1
             join images_product i on i.product_id = p.id
             where ps.seller_id = s.id
             order by p.name asc, p.id asc, i.is_main desc nulls last, i.display_order asc, i.id asc
             limit 1) as image_url
        from seller s
        where s.status = 1
        order by f_unaccent(lower(s.store_name)) asc, s.id asc
    "#)
        .load::<StoreListItem>(connection).await
}

/// Las `limit` tiendas con más productos a la venta (empate: por nombre, que
/// ya traen de `list_active_stores`). Sin las vacías. Para la franja de la portada.
pub fn biggest_stores(mut stores: Vec<StoreListItem>, limit: usize) -> Vec<StoreListItem> {
    stores.retain(|store| store.products > 0);
    // sort_by es estable: entre iguales se queda el orden por nombre.
    stores.sort_by(|a, b| b.products.cmp(&a.products));
    stores.truncate(limit);
    stores
}

/// Las tiendas con productos cuyo nombre encaja con `q` (como las categorías
/// en las sugerencias del buscador), en el orden en que llegan.
pub fn stores_named_like(stores: Vec<StoreListItem>, q: &str, limit: usize) -> Vec<StoreListItem> {
    stores.into_iter()
        .filter(|store| store.products > 0 && name_matches(&store.store_name, q))
        .take(limit)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(name: &str, products: i64) -> StoreListItem {
        StoreListItem { id: Uuid::nil(), store_name: name.into(), products, image_url: None }
    }

    fn names(stores: &[StoreListItem]) -> Vec<&str> {
        stores.iter().map(|s| s.store_name.as_str()).collect()
    }

    /// Las de más productos primero; las vacías fuera; a igualdad, el orden de llegada.
    #[test]
    fn biggest_stores_skip_empty_ones_and_keep_name_order_on_ties() {
        let all = vec![store("Atelier", 5), store("Botica", 9), store("Casa", 0), store("Dúo", 5), store("Eco", 1)];
        assert_eq!(names(&biggest_stores(all.clone(), 3)), ["Botica", "Atelier", "Dúo"]);
        assert_eq!(names(&biggest_stores(all, 10)), ["Botica", "Atelier", "Dúo", "Eco"]);
    }

    /// Sin acentos ni mayúsculas, palabras cortas al principio y sin las vacías.
    #[test]
    fn stores_named_like_matches_like_the_categories() {
        let all = vec![store("Casa Nómada", 3), store("Casa Molushop", 0), store("Moda Molushop", 2)];
        assert_eq!(names(&stores_named_like(all.clone(), "NOMADA", 3)), ["Casa Nómada"]);
        assert_eq!(names(&stores_named_like(all.clone(), "mo", 3)), ["Moda Molushop"], "vacía, fuera");
        assert!(stores_named_like(all.clone(), "da", 3).is_empty(), "corta, solo al principio de palabra");
        assert_eq!(names(&stores_named_like(all, "casa", 3)), ["Casa Nómada"]);
    }
}
