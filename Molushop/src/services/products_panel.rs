// logica de implementación
// obtener productos y variaciones
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::result::Error;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use crate::models::data_transfer_objects::product::Product;
use crate::models::db::{ProductVariation,Price,Products};
use crate::models::data_transfer_objects::product;
use uuid::Uuid;
type DbPool = Pool<AsyncPgConnection>;

use diesel::sql_query;
use diesel::sql_types::{Array, Nullable, SmallInt, Text, Uuid as SqlUuid};
use std::collections::HashMap;
use diesel_async::{AsyncConnection, RunQueryDsl};
use crate::constants::product_status::PRODUCT_DELETED;
use crate::utils::db::get_connection;

/// "Eliminar" del panel. No borra la fila (los pedidos y las opiniones apuntan
/// a sus variaciones): la marca como `PRODUCT_DELETED` y, en la misma
/// transacción, la quita de los carritos activos, para que nadie se quede con
/// una línea que ya no puede comprar, y libera los SKU de sus variaciones.
///
/// `store_ids`: `None` (el admin) cualquier producto; `Some(tiendas)` solo si
/// es de una de ellas. `Ok(false)` si no existe, ya estaba borrado o es ajeno.
pub async fn delete_product(product_id: &Uuid, store_ids: Option<&[Uuid]>, pool: &DbPool) -> Result<bool, Error> {
    let mut conn = get_connection(pool).await?;
    let store_ids: Option<Vec<Uuid>> = store_ids.map(|ids| ids.to_vec());

    conn.transaction::<bool, Error, _>(async move |conn| {
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

        // Su SKU queda libre para otra variación de la tienda (los pedidos
        // guardan su propia copia).
        sql_query("UPDATE product_variations SET sku = NULL WHERE product_id = $1 AND sku IS NOT NULL")
            .bind::<SqlUuid, _>(product_id)
            .execute(conn)
            .await?;
        Ok(true)
    }).await
}

/// Productos de una tienda para su listado del panel. Quien llama ya ha
/// comprobado que el usuario puede entrar en esa tienda.
///
/// Todo por lotes y con una sola conexión: los productos, sus imágenes y
/// `load_panel_extras` son 5 consultas para todo el listado (antes, 3 más por producto).
pub async fn get_products(store_id:&Uuid,pool: &DbPool) -> Result<Vec<Product>,Error>{
    let connection = &mut get_connection(pool).await?;
    let products = get_panel_products(Some(std::slice::from_ref(store_id)), connection).await?;
    let product_ids: Vec<Uuid> = products.iter().map(|(p, _)| p.id).collect();
    let mut images = get_main_images(&product_ids, connection).await?;
    let mut extras = load_panel_extras(&product_ids, PANEL_CURRENCY, connection).await?;

    Ok(products
        .into_iter()
        .map(|(product, _store_name)| {
            let image_url = images.remove(&product.id);
            to_panel_product(product, store_id, image_url, &mut extras).0
        })
        .collect())
}

/// Un producto con las mismas variaciones que el listado (el explorador de
/// variaciones). Quien llama ya ha comprobado que puede verlo.
pub async fn get_product(store_id:&Uuid, product_id:&Uuid, pool: &DbPool) -> Result<Product,Error>{
    use crate::schema::products;
    let connection = &mut get_connection(pool).await?;
    let product = products::table.find(product_id).first::<Products>(connection).await?;
    let image_url = get_main_images(std::slice::from_ref(product_id), connection).await?.remove(product_id);
    Ok(panel_product_on(connection, store_id, product, image_url).await?.0)
}

/// `product` como lo enseña el panel, con la conexión que se le da (3
/// consultas: categorías, variaciones y precios). Devuelve también las filas de
/// sus variaciones, para quien las necesite enteras (el editor) sin volver a leerlas.
pub async fn panel_product_on(
    connection: &mut AsyncPgConnection,
    store_id: &Uuid,
    product: Products,
    image_url: Option<String>,
) -> Result<(Product, Vec<ProductVariation>), Error> {
    let mut extras = load_panel_extras(std::slice::from_ref(&product.id), PANEL_CURRENCY, connection).await?;
    Ok(to_panel_product(product, store_id, image_url, &mut extras))
}

/// El panel enseña los precios en euros.
const PANEL_CURRENCY: &str = "EUR";

/// Categorías, variaciones y precios de varios productos, ya repartidos.
#[derive(Default)]
struct PanelExtras {
    /// Nombres de las categorías, por producto.
    categories: HashMap<Uuid, Vec<String>>,
    /// Todas las variaciones, por producto.
    variations: HashMap<Uuid, Vec<ProductVariation>>,
    /// El precio en `PANEL_CURRENCY`, por variación.
    prices: HashMap<Uuid, Price>,
}

/// Una consulta por tabla para todos los productos (`= ANY`).
async fn load_panel_extras(product_ids: &[Uuid], currency: &str, connection: &mut AsyncPgConnection) -> Result<PanelExtras, Error> {
    use crate::schema::{category, category_product, prices, product_variations as pv};
    if product_ids.is_empty() {
        return Ok(PanelExtras::default());
    }

    let category_rows: Vec<(Uuid, String)> = category::table
        .inner_join(category_product::table.on(category::id.eq(category_product::category_id)))
        .filter(category_product::product_id.eq_any(product_ids))
        .select((category_product::product_id, category::name))
        .load(connection)
        .await?;

    let variation_rows: Vec<ProductVariation> = pv::table
        .filter(pv::product_id.eq_any(product_ids))
        .load(connection)
        .await?;

    let variation_ids: Vec<Uuid> = variation_rows.iter().map(|v| v.id).collect();
    let price_rows: Vec<Price> = prices::table
        .filter(prices::variation_id.eq_any(&variation_ids))
        .filter(prices::currency.eq(currency))
        .load(connection)
        .await?;

    let mut extras = PanelExtras::default();
    for (product_id, name) in category_rows {
        extras.categories.entry(product_id).or_default().push(name);
    }
    for variation in variation_rows {
        extras.variations.entry(variation.product_id).or_default().push(variation);
    }
    for price in price_rows {
        extras.prices.insert(price.variation_id, price);
    }
    Ok(extras)
}

/// Un producto con todas sus variaciones, también las que no tienen precio en
/// EUR (salen "No disponible"; antes el `inner_join` con `prices` las
/// escondía y no había forma de arreglarlas desde el panel). Devuelve también
/// las filas de las variaciones.
fn to_panel_product(product: Products, store_id:&Uuid, image_url: Option<String>, extras: &mut PanelExtras) -> (Product, Vec<ProductVariation>){
    let currency = PANEL_CURRENCY.to_string();
    let categories_names: Vec<String> = extras.categories.remove(&product.id).unwrap_or_default();
    let vars: Vec<ProductVariation> = extras.variations.remove(&product.id).unwrap_or_default();

    let variation_titles: Vec<String> =
        crate::utils::db_json::parse_db_json::<VariationTitles>(product.variation_titles, "variation_titles", &product.id)
            .map(|titles| titles.titulos)
            .unwrap_or_default();
    let variations: Vec<product::ProductVariation> = vars.iter()
        .map(|pv| product::ProductVariation::from(pv, extras.prices.get(&pv.id), &currency))
        .collect();

    (Product{
        variations: variations,
        name: product.name,
        status: product.status,
        code: product.code,
        variations_titles: variation_titles,
        categories: categories_names,
        id: product.id,
        store_name: None,
        store_id: *store_id,
        image_url,
    }, vars)
}

#[derive(diesel::QueryableByName)]
struct MainImage {
    #[diesel(sql_type = SqlUuid)]
    product_id: Uuid,
    #[diesel(sql_type = Text)]
    image_url: String,
}

/// Cuál es la imagen principal de un producto: la marcada `is_main` y si no,
/// la primera por `display_order`. `NULLS LAST` porque en orden descendente
/// Postgres pone los NULL delante de los `true`. La usa también el editor.
pub const MAIN_IMAGE_ORDER: &str = "is_main DESC NULLS LAST, display_order ASC, id ASC";

/// La imagen principal de cada producto, en una consulta para todo el listado.
async fn get_main_images(product_ids: &[Uuid], connection: &mut AsyncPgConnection) -> Result<HashMap<Uuid, String>, Error> {
    if product_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let rows = sql_query(format!(
        "SELECT DISTINCT ON (product_id) product_id, image_url
         FROM images_product WHERE product_id = ANY($1)
         ORDER BY product_id, {MAIN_IMAGE_ORDER}"
    ))
        .bind::<Array<SqlUuid>, _>(product_ids)
        .load::<MainImage>(connection)
        .await?;
    Ok(rows.into_iter().map(|r| (r.product_id, r.image_url)).collect())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariationTitles{
    pub titulos: Vec<String>,
}

// obtener los productos de un vendedor 

/// Productos del panel con el nombre de su tienda. `None` = todas las tiendas
/// (admin); `Some(tiendas)` = solo las de esas tiendas. Los productos sin fila en
/// `product_seller` (los "borrados") no salen.
pub async fn get_panel_products(seller_filter:Option<&[Uuid]>,connection: &mut AsyncPgConnection) -> Result<Vec<(Products,String)>,Error>{
    use crate::schema::{product_seller,products,seller};
    let mut query = products::table
        .inner_join(product_seller::table.inner_join(seller::table))
        .select((Products::as_select(), seller::store_name))
        // Lo "eliminado" desde el panel ya no se lista.
        .filter(products::status.ne(crate::constants::product_status::PRODUCT_DELETED))
        .order_by((seller::store_name, products::name))
        .into_boxed();
    if let Some(store_ids) = seller_filter {
        query = query.filter(product_seller::seller_id.eq_any(store_ids));
    }
    query.load::<(Products,String)>(connection).await
}
