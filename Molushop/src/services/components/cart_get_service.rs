use diesel::sql_query;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use uuid::Uuid;
use askama::Template;

use crate::models::models_x::CartItem1;
use crate::models::components::cart_total_v1::{CartTotalV1, CartTotalV1Body, CartTotalData, CartItemData};

type DbPool = Pool<AsyncPgConnection>;

/// Devuelve las líneas del carrito ACTIVO (status = 1) del usuario, ya resueltas
/// con nombre, variación, imagen, precio actual, tienda y stock disponible.
pub async fn get_cart_items(user_id: &Uuid, pool: &DbPool) -> Result<Vec<CartItem1>, Error> {
    let connection = &mut pool.get().await.unwrap();
    let results = sql_query(r#"
        SELECT
            cp.id,
            cp.product_var_id,
            p.id AS product_id,
            p.name,
            (
                SELECT string_agg(attr->>'value', ' / ')
                FROM jsonb_array_elements(pv.attributes) AS attr
            ) AS variation_label,
            COALESCE(
                (SELECT img.image_url
                 FROM images_product_variations ipv
                 JOIN images_product img ON img.id = ipv.image_id
                 WHERE ipv.variation_id = pv.id
                 ORDER BY img.is_main DESC, img.display_order ASC
                 LIMIT 1),
                (SELECT img.image_url
                 FROM images_product img
                 WHERE img.product_id = p.id
                 ORDER BY img.is_main DESC, img.display_order ASC
                 LIMIT 1)
            ) AS image_url,
            pr.price,
            pr.currency,
            s.store_name,
            cp.quantity,
            pv.stock,
            (pr.price * cp.quantity) AS subtotal
        FROM cart_products cp
        JOIN carts c ON c.id = cp.cart_id
        JOIN product_variations pv ON pv.id = cp.product_var_id
        JOIN products p ON p.id = pv.product_id
        LEFT JOIN LATERAL (
            SELECT pr2.price, pr2.currency
            FROM prices pr2
            WHERE pr2.variation_id = pv.id
            ORDER BY pr2.start_date DESC
            LIMIT 1
        ) pr ON true
        LEFT JOIN LATERAL (
            SELECT s2.store_name
            FROM product_seller ps
            JOIN seller s2 ON s2.id = ps.seller_id
            WHERE ps.product_id = p.id
            LIMIT 1
        ) s ON true
        WHERE c.user_id = $1 AND c.status = 1
        ORDER BY cp.added_at ASC
    "#).bind::<diesel::sql_types::Uuid, _>(user_id)
        .load::<CartItem1>(connection).await?;
    Ok(results)
}

/// Objeto de vista del carrito. Si el usuario no está logueado o falla la consulta
/// se devuelve un carrito vacío en lugar de un error, igual que el resto de componentes.
pub async fn get_cart_total_object(user_id: Option<&Uuid>, pool: &DbPool) -> CartTotalData {
    let uid = match user_id {
        Some(u) => u,
        None => return CartTotalData::default(),
    };

    let rows = match get_cart_items(uid, pool).await {
        Ok(r) => r,
        Err(e) => {
            println!("Error obteniendo el carrito: {:?}", e);
            return CartTotalData::default();
        }
    };

    let items: Vec<CartItemData> = rows.into_iter().map(CartItemData::from).collect();
    CartTotalData::from_items(items)
}

pub async fn get_cart_total_render(user_id: Option<&Uuid>, pool: &DbPool) -> String {
    let objeto = get_cart_total_object(user_id, pool).await;
    CartTotalV1 {
        cart_total: objeto
    }.render().unwrap()
}

/// Render del cuerpo del componente, para los refrescos parciales tras
/// modificar o eliminar una línea.
pub async fn get_cart_total_body_render(user_id: Option<&Uuid>, pool: &DbPool) -> String {
    let objeto = get_cart_total_object(user_id, pool).await;
    CartTotalV1Body {
        cart_total: objeto
    }.render().unwrap()
}
