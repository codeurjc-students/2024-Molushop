use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use uuid::Uuid;

use crate::schema::customer_address;
use crate::models::models_x::CustomerAddress;
use crate::models::components::cart_total_v1::{CartTotalData, CartItemData};
use crate::models::components::checkout_v1::{CheckoutData, AddressData, ROUTES};
use crate::services::components::cart_get_service::get_cart_items;

type DbPool = Pool<AsyncPgConnection>;

/// Traduce el `?lines=id1,id2` a un conjunto de UUIDs. Lo que no sea un UUID se
/// descarta sin más: el filtrado posterior es contra las líneas del usuario, así
/// que un id inventado o de otra persona simplemente no casa con nada.
fn parse_line_ids(lines: Option<&str>) -> Option<Vec<Uuid>> {
    let raw = lines?.trim();
    if raw.is_empty() { return None; }

    let ids: Vec<Uuid> = raw
        .split(',')
        .filter_map(|part| Uuid::parse_str(part.trim()).ok())
        .collect();

    Some(ids)
}

/// Primera dirección guardada del usuario, para dejar el formulario relleno.
/// Elegir entre varias es mejora futura; de momento se usa la primera.
async fn get_first_address(user_id: &Uuid, pool: &DbPool) -> AddressData {
    let connection = &mut match pool.get().await {
        Ok(c) => c,
        Err(e) => {
            println!("Error obteniendo conexión para la dirección: {:?}", e);
            return AddressData::default();
        }
    };

    let row = customer_address::table
        .filter(customer_address::customer_id.eq(user_id))
        .first::<CustomerAddress>(connection)
        .await;

    match row {
        Ok(a) => AddressData {
            street1: a.street1,
            street2: a.street2.unwrap_or_default(),
            postal_code: a.postal_code,
            city: a.city,
            province: a.province
        },
        // Sin dirección guardada no hay error que reportar: el formulario sale vacío
        Err(_) => AddressData::default()
    }
}

/// Objeto de vista del checkout. Si el usuario no está logueado o falla la
/// consulta se devuelve un checkout vacío, igual que hace el carrito.
///
/// `lines` es el `?lines=` de la URL: sin él se compra el carrito entero.
pub async fn get_checkout_object(
    user_id: Option<&Uuid>,
    lines: Option<&str>,
    pool: &DbPool
) -> CheckoutData {
    let uid = match user_id {
        Some(u) => u,
        None => return CheckoutData::default(),
    };

    let rows = match get_cart_items(uid, pool).await {
        Ok(r) => r,
        Err(e) => {
            println!("Error obteniendo el carrito para el checkout: {:?}", e);
            return CheckoutData::default();
        }
    };

    let mut items: Vec<CartItemData> = rows.into_iter().map(CartItemData::from).collect();

    // El filtro se aplica sobre las líneas que YA son del usuario, así que un id
    // de otra persona no puede colarse por aquí
    if let Some(wanted) = parse_line_ids(lines) {
        items.retain(|it| wanted.contains(&it.id));
    }

    let line_ids = items
        .iter()
        .map(|it| it.id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    CheckoutData {
        routes: &ROUTES,
        cart_total: CartTotalData::from_items(items),
        address: get_first_address(uid, pool).await,
        line_ids
    }
}
