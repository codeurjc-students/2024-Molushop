// logica de implementación
// obtener productos y variaciones
use serde::{Serialize, Deserialize};
use diesel::result::Error;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use crate::models::data_transfer_objects::product::Product;
use crate::models::models_x::{ProductVariation,Category,Price};
use crate::models::data_transfer_objects::product;
//use crate::schema::products::variations;
use uuid::Uuid;
use crate::services::servicesX::{
    get_panel_products,
    get_product_categories,
    get_product_variations,
    get_variations_with_prices
};
type DbPool = Pool<AsyncPgConnection>;

use diesel::sql_query;
use diesel::sql_types::{Array, Nullable, SmallInt, Text, Uuid as SqlUuid};
use std::collections::HashMap;
use diesel_async::{AsyncConnection, RunQueryDsl};
use diesel_async::scoped_futures::ScopedFutureExt;
use crate::constants::product_status::PRODUCT_DELETED;

/// "Eliminar" del panel. No borra la fila (los pedidos y las opiniones apuntan
/// a sus variaciones): la marca como `PRODUCT_DELETED` y, en la misma
/// transacción, la quita de los carritos activos, para que nadie se quede con
/// una línea que ya no puede comprar.
///
/// `store_ids`: `None` (el admin) cualquier producto; `Some(tiendas)` solo si
/// es de una de ellas. `Ok(false)` si no existe, ya estaba borrado o es ajeno.
pub async fn delete_product(product_id: &Uuid, store_ids: Option<&[Uuid]>, pool: &DbPool) -> Result<bool, Error> {
    let mut conn = pool.get().await.unwrap();
    let store_ids: Option<Vec<Uuid>> = store_ids.map(|ids| ids.to_vec());

    conn.transaction::<bool, Error, _>(|conn| async move {
        let marked = sql_query(
            "UPDATE products SET status = $1              WHERE id = $2 AND status <> $1                AND ($3::uuid[] IS NULL OR EXISTS (                    SELECT 1 FROM product_seller ps                    WHERE ps.product_id = products.id AND ps.seller_id = ANY($3)))"
        )
            .bind::<SmallInt, _>(PRODUCT_DELETED)
            .bind::<SqlUuid, _>(product_id)
            .bind::<Nullable<Array<SqlUuid>>, _>(store_ids)
            .execute(conn)
            .await?;
        if marked == 0 {
            return Ok(false);
        }

        sql_query(
            "DELETE FROM cart_products cp              USING carts c, product_variations pv              WHERE c.id = cp.cart_id AND c.status = 1                AND pv.id = cp.product_var_id AND pv.product_id = $1"
        )
            .bind::<SqlUuid, _>(product_id)
            .execute(conn)
            .await?;
        Ok(true)
    }.scope_boxed()).await
}

/// Productos de una tienda para su listado del panel. Quien llama ya ha
/// comprobado que el usuario puede entrar en esa tienda.
pub async fn get_products(store_id:&Uuid,pool: &DbPool) -> Result<Vec<Product>,Error>{

    let products = get_panel_products(Some(std::slice::from_ref(store_id)), pool).await?;
    let product_ids: Vec<Uuid> = products.iter().map(|(p, _)| p.id).collect();
    let mut images = get_main_images(&product_ids, pool).await?;
    let mut products_final:Vec<Product> = Vec::new();

    for (product, _store_name) in products{
        let categories: Vec<Category> = get_product_categories(&product.id, &pool).await?;
        let categories_names:Vec<String> = categories.iter().map(|c| c.name.clone()).collect();
        let vars: Vec<ProductVariation> = get_product_variations(&product.id, &pool).await?;
        let vars_price: Vec<(ProductVariation,Price)> = get_variations_with_prices(&product.id, &"EUR".to_string(), &pool).await?;
        //let variations:Vec<product::ProductVariation> = Vec::new();
        
        let variation_titles:Vec<String> = match product.variation_titles{
            Some(titles) =>{
                let titles_aux:VariationTitles = serde_json::from_value(titles).unwrap();
                titles_aux.titulos
            },
            None => Vec::new()     
        };
        //let variations: Vec<product::ProductVariation> = vars.iter().map(|v| product::ProductVariation::from(v)).collect();  
        let variations: Vec<product::ProductVariation> = vars_price.iter().map(|(pv,p)| product::ProductVariation::from(pv,p)).collect();  

        let product_final = Product{
            variations: variations,
            name: product.name,
            status: product.status,
            code: product.code,
            variations_titles: variation_titles,
            categories: categories_names,
            id: product.id,
            store_name: None,
            store_id: *store_id,
            image_url: images.remove(&product.id),
        };
        products_final.push(product_final);
    }
        
    Ok(products_final)
}

#[derive(diesel::QueryableByName)]
struct MainImage {
    #[diesel(sql_type = SqlUuid)]
    product_id: Uuid,
    #[diesel(sql_type = Text)]
    image_url: String,
}

/// La imagen principal de cada producto, en una consulta para todo el listado.
/// La marcada `is_main` y si no, la primera por `display_order`. `NULLS LAST`
/// porque en orden descendente Postgres pone los NULL delante de los `true`.
async fn get_main_images(product_ids: &[Uuid], pool: &DbPool) -> Result<HashMap<Uuid, String>, Error> {
    if product_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let mut conn = pool.get().await.unwrap();
    let rows = sql_query(
        "SELECT DISTINCT ON (product_id) product_id, image_url
         FROM images_product WHERE product_id = ANY($1)
         ORDER BY product_id, is_main DESC NULLS LAST, display_order ASC, id ASC"
    )
        .bind::<Array<SqlUuid>, _>(product_ids)
        .load::<MainImage>(&mut conn)
        .await?;
    Ok(rows.into_iter().map(|r| (r.product_id, r.image_url)).collect())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariationTitles{
    pub titulos: Vec<String>,
}


// obtener los productos de un vendedor 