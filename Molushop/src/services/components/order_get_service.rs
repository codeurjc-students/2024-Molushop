use std::collections::HashMap;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use uuid::Uuid;

use crate::schema::{orders, order_items, product_variations};
use crate::models::models_x::{Order, OrderItem};
use crate::models::components::order_detail_v1::OrderData;
use crate::models::components::order_list_v1::OrderListData;

type DbPool = Pool<AsyncPgConnection>;

/// Ficha de un pedido. El filtro por `user_id` va en la propia consulta: es lo
/// que impide leer el pedido de otro poniendo su UUID en la URL.
///
/// Si no existe o no es suyo se devuelve un `OrderData` con `found = false`, no
/// un error: la página enseña un aviso, igual que hace el carrito cuando está
/// vacío. Sin sesión va además con `logged = false` y la página pide identificarse.
pub async fn get_order_object(
    user_id: Option<&Uuid>,
    order_id: &Uuid,
    pool: &DbPool
) -> OrderData {
    let uid = match user_id {
        Some(u) => u,
        None => return OrderData::default(),
    };

    let connection = &mut match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            println!("Error obteniendo conexión para el pedido: {:?}", e);
            return OrderData::not_found_logged();
        }
    };

    let order: Order = match orders::table
        .filter(orders::id.eq(order_id))
        .filter(orders::user_id.eq(uid))
        .first::<Order>(connection)
        .await
    {
        Ok(o) => o,
        Err(_) => return OrderData::not_found_logged(),
    };

    let items: Vec<OrderItem> = match order_items::table
        .filter(order_items::order_id.eq(&order.id))
        .load::<OrderItem>(connection)
        .await
    {
        Ok(i) => i,
        Err(e) => {
            println!("Error obteniendo las líneas del pedido: {:?}", e);
            Vec::new()
        }
    };

    // El producto de cada variación, para enlazar las líneas a su ficha. La
    // variación siempre existe (la FK de order_items es ON DELETE RESTRICT); si
    // la consulta falla, las líneas salen igual, solo que sin enlace.
    let var_ids: Vec<Uuid> = items.iter().map(|it| it.product_var_id).collect();
    let product_ids: HashMap<Uuid, Uuid> = match product_variations::table
        .filter(product_variations::id.eq_any(&var_ids))
        .select((product_variations::id, product_variations::product_id))
        .load::<(Uuid, Uuid)>(connection)
        .await
    {
        Ok(rows) => rows.into_iter().collect(),
        Err(e) => {
            println!("Error obteniendo los productos de las líneas del pedido: {:?}", e);
            HashMap::new()
        }
    };

    OrderData::from_db(order, items, &product_ids)
}

/// "Mis pedidos": todos los del usuario, del más reciente al más antiguo.
///
/// Son DOS consultas, no una por pedido: las líneas de todos se traen de golpe
/// con un `order_id = ANY(...)` y se reparten aquí. Con la ficha individual daba
/// igual, pero aquí el nº de pedidos crece con el tiempo y un N+1 crecería con él.
///
/// Sin sesión devuelve la lista vacía con `logged = false`, no un error: la
/// página pide identificarse y abre el login.
pub async fn get_orders_list_object(
    user_id: Option<&Uuid>,
    pool: &DbPool
) -> OrderListData {
    let uid = match user_id {
        Some(u) => u,
        None => return OrderListData::default(),
    };

    let connection = &mut match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            println!("Error obteniendo conexión para la lista de pedidos: {:?}", e);
            return OrderListData::empty_logged();
        }
    };

    let orders_db: Vec<Order> = match orders::table
        .filter(orders::user_id.eq(uid))
        .order(orders::created_at.desc())
        .load::<Order>(connection)
        .await
    {
        Ok(o) => o,
        Err(e) => {
            println!("Error obteniendo los pedidos del usuario: {:?}", e);
            return OrderListData::empty_logged();
        }
    };

    if orders_db.is_empty() {
        return OrderListData::empty_logged();
    }

    let ids: Vec<Uuid> = orders_db.iter().map(|o| o.id).collect();

    let items_db: Vec<OrderItem> = match order_items::table
        .filter(order_items::order_id.eq_any(&ids))
        .load::<OrderItem>(connection)
        .await
    {
        Ok(i) => i,
        Err(e) => {
            println!("Error obteniendo las líneas de los pedidos: {:?}", e);
            Vec::new()
        }
    };

    // Se agrupan por pedido conservando el orden en que los devolvió la BD, que
    // es el mismo criterio (ninguno) que usa la ficha individual.
    let mut by_order: HashMap<Uuid, Vec<OrderItem>> = HashMap::new();
    for item in items_db {
        by_order.entry(item.order_id).or_default().push(item);
    }

    let rows: Vec<(Order, Vec<OrderItem>)> = orders_db.into_iter()
        .map(|o| {
            let items = by_order.remove(&o.id).unwrap_or_default();
            (o, items)
        })
        .collect();

    OrderListData::from_db(rows)
}
