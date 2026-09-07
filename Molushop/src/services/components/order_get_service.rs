use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use uuid::Uuid;

use crate::schema::{orders, order_items};
use crate::models::models_x::{Order, OrderItem};
use crate::models::components::order_detail_v1::OrderData;

type DbPool = Pool<AsyncPgConnection>;

/// Ficha de un pedido. El filtro por `user_id` va en la propia consulta: es lo
/// que impide leer el pedido de otro poniendo su UUID en la URL.
///
/// Si no existe o no es suyo se devuelve un `OrderData` con `found = false`, no
/// un error: la página enseña un aviso, igual que hace el carrito cuando está
/// vacío.
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
            return OrderData::default();
        }
    };

    let order: Order = match orders::table
        .filter(orders::id.eq(order_id))
        .filter(orders::user_id.eq(uid))
        .first::<Order>(connection)
        .await
    {
        Ok(o) => o,
        Err(_) => return OrderData::default(),
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

    OrderData::from_db(order, items)
}
