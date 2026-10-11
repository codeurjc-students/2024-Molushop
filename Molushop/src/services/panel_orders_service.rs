//! Pedidos en el panel: listarlos, ver uno y cambiar el estado de sus líneas.
//!
//! Todo va con un `scope` opcional de tienda. `Some(tienda)`: solo los pedidos
//! con líneas de esa tienda y solo esas líneas (`/panel/stores/{id}/orders`).
//! `None`: todos los pedidos con todas sus líneas (el admin).
//!
//! El estado se guarda POR LÍNEA (`order_items.status`): un pedido puede mezclar
//! tiendas y cada una envía lo suyo. `orders.status` es el resumen de sus líneas
//! y solo lo escribe `refresh_order_status`.

use std::collections::HashMap;

use bigdecimal::BigDecimal;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::sql_query;
use diesel::sql_types::{Integer, Nullable, SmallInt, Uuid as SqlUuid};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::constants::money::symbol_of;
use crate::models::components::order_detail_v1::{
    status_label, ORDER_CANCELLED, ORDER_CONFIRMED, ORDER_DELIVERED, ORDER_SHIPPED,
};
use crate::models::models_x::{Order, OrderItem};
use crate::schema::{base_user, order_items, orders};

type DbPool = Pool<AsyncPgConnection>;

#[derive(Debug)]
pub enum PanelOrderError {
    /// La línea no está en el estado del que parte el cambio. También si no
    /// existe o es de otra tienda: no se distingue para no revelar líneas ajenas.
    InvalidTransition,
    Db(String),
}

impl PanelOrderError {
    /// Código corto para llevar el error en la URL tras el POST (`?error=...`).
    pub fn code(&self) -> &'static str {
        match self {
            PanelOrderError::InvalidTransition => "transition",
            PanelOrderError::Db(_) => "db",
        }
    }
}

impl From<DieselError> for PanelOrderError {
    fn from(e: DieselError) -> Self {
        PanelOrderError::Db(format!("{:?}", e))
    }
}

async fn connection(
    pool: &DbPool,
) -> Result<diesel_async::pooled_connection::deadpool::Object<AsyncPgConnection>, PanelOrderError> {
    pool.get()
        .await
        .map_err(|e| PanelOrderError::Db(format!("Error obteniendo conexión: {:?}", e)))
}

/// Estado del que tiene que partir una línea para pasar a `new_status`:
/// confirmada -> enviada -> entregada, y cancelar solo una confirmada. No hay
/// marcha atrás. `None`: a ese estado no se llega desde el panel.
fn required_status(new_status: i16) -> Option<i16> {
    match new_status {
        ORDER_SHIPPED => Some(ORDER_CONFIRMED),
        ORDER_DELIVERED => Some(ORDER_SHIPPED),
        ORDER_CANCELLED => Some(ORDER_CONFIRMED),
        _ => None,
    }
}

/// Resumen de un grupo de líneas: el estado de la más atrasada de las que no
/// están canceladas (0 < 1 < 3 < 4 ya van en ese orden), o cancelado si lo
/// están todas. Es la misma regla que aplica `refresh_order_status` en SQL.
fn summary_status(lines: &[PanelOrderLine]) -> i16 {
    lines
        .iter()
        .map(|l| l.status)
        .filter(|s| *s != ORDER_CANCELLED)
        .min()
        .unwrap_or(ORDER_CANCELLED)
}

/// Una línea del pedido tal como se pinta en el panel (snapshot de `order_items`).
#[derive(Debug, Clone)]
pub struct PanelOrderLine {
    pub id: Uuid,
    pub product_name: String,
    pub variation_label: String,
    pub sku: String,
    pub store_name: String,
    pub unit_price: BigDecimal,
    pub quantity: i32,
    pub line_total: BigDecimal,
    pub status: i16,
}

impl PanelOrderLine {
    pub fn status_label(&self) -> &'static str {
        status_label(self.status)
    }

    pub fn can_ship(&self) -> bool {
        required_status(ORDER_SHIPPED) == Some(self.status)
    }

    pub fn can_deliver(&self) -> bool {
        required_status(ORDER_DELIVERED) == Some(self.status)
    }

    pub fn can_cancel(&self) -> bool {
        required_status(ORDER_CANCELLED) == Some(self.status)
    }
}

impl From<OrderItem> for PanelOrderLine {
    fn from(db: OrderItem) -> Self {
        Self {
            id: db.id,
            product_name: db.product_name,
            variation_label: db.variation_label.unwrap_or_default(),
            sku: db.sku.unwrap_or_default(),
            store_name: db.store_name.unwrap_or_default(),
            unit_price: db.unit_price,
            quantity: db.quantity,
            line_total: db.line_total,
            status: db.status,
        }
    }
}

/// Un pedido en el panel, con las líneas que el `scope` deja ver.
#[derive(Debug, Clone)]
pub struct PanelOrder {
    pub id: Uuid,
    pub order_number: i64,
    /// Fecha del pedido, ya formateada (dd/mm/aaaa).
    pub created: String,
    /// Username de quien compró.
    pub buyer: String,
    pub currency: String,
    pub ship_street1: String,
    pub ship_street2: String,
    pub ship_postal_code: String,
    pub ship_city: String,
    pub ship_province: String,
    pub lines: Vec<PanelOrderLine>,
}

impl PanelOrder {
    fn from_db(order: Order, buyer: String, items: Vec<OrderItem>) -> Self {
        Self {
            id: order.id,
            order_number: order.order_number,
            created: order.created_at.format("%d/%m/%Y").to_string(),
            buyer,
            currency: order.currency.trim().to_string(),
            ship_street1: order.ship_street1,
            ship_street2: order.ship_street2.unwrap_or_default(),
            ship_postal_code: order.ship_postal_code,
            ship_city: order.ship_city,
            ship_province: order.ship_province,
            lines: items.into_iter().map(PanelOrderLine::from).collect(),
        }
    }

    pub fn currency_symbol(&self) -> String {
        symbol_of(&self.currency)
    }

    /// Estado de las líneas que se ven. Con todas las líneas coincide con
    /// `orders.status`; para una tienda es el estado de SU parte del pedido.
    pub fn status(&self) -> i16 {
        summary_status(&self.lines)
    }

    pub fn status_label(&self) -> &'static str {
        status_label(self.status())
    }

    /// Nº de unidades, no de líneas.
    pub fn total_units(&self) -> i32 {
        self.lines.iter().map(|l| l.quantity).sum()
    }

    /// Importe de las líneas que se ven (para una tienda, lo suyo; no el total
    /// del pedido). Las canceladas no suman.
    pub fn lines_total(&self) -> BigDecimal {
        self.lines
            .iter()
            .filter(|l| l.status != ORDER_CANCELLED)
            .map(|l| &l.line_total)
            .sum()
    }
}

/// Los pedidos, del más reciente al más antiguo, y `None` si el pedido pedido no
/// entra en el `scope`. Dos consultas (pedidos y líneas), no una por pedido.
async fn load_orders(
    scope: Option<&Uuid>,
    only: Option<&Uuid>,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<PanelOrder>, PanelOrderError> {
    let mut orders_query = orders::table
        .inner_join(base_user::table)
        .select((orders::all_columns, base_user::username))
        .order(orders::created_at.desc())
        .into_boxed();
    if let Some(store_id) = scope {
        orders_query = orders_query.filter(
            orders::id.eq_any(
                order_items::table
                    .filter(order_items::store_id.eq(store_id))
                    .select(order_items::order_id),
            ),
        );
    }
    if let Some(order_id) = only {
        orders_query = orders_query.filter(orders::id.eq(order_id));
    }
    let orders_db = orders_query.load::<(Order, String)>(conn).await?;

    let ids: Vec<Uuid> = orders_db.iter().map(|(o, _)| o.id).collect();
    let mut items_query = order_items::table
        .filter(order_items::order_id.eq_any(&ids))
        .order((order_items::product_name, order_items::id))
        .into_boxed();
    if let Some(store_id) = scope {
        items_query = items_query.filter(order_items::store_id.eq(store_id));
    }
    let items_db = items_query.load::<OrderItem>(conn).await?;

    let mut by_order: HashMap<Uuid, Vec<OrderItem>> = HashMap::new();
    for item in items_db {
        by_order.entry(item.order_id).or_default().push(item);
    }

    Ok(orders_db
        .into_iter()
        .map(|(order, buyer)| {
            let items = by_order.remove(&order.id).unwrap_or_default();
            PanelOrder::from_db(order, buyer, items)
        })
        .collect())
}

pub async fn list_orders(scope: Option<&Uuid>, pool: &DbPool) -> Result<Vec<PanelOrder>, PanelOrderError> {
    let conn = &mut connection(pool).await?;
    load_orders(scope, None, conn).await
}

/// `Ok(None)` si el pedido no existe o no tiene líneas de la tienda del `scope`:
/// el handler responde 404 en los dos casos.
pub async fn get_order(
    scope: Option<&Uuid>,
    order_id: &Uuid,
    pool: &DbPool,
) -> Result<Option<PanelOrder>, PanelOrderError> {
    let conn = &mut connection(pool).await?;
    Ok(load_orders(scope, Some(order_id), conn).await?.into_iter().next())
}

#[derive(QueryableByName)]
struct ChangedLine {
    #[diesel(sql_type = SqlUuid)]
    product_var_id: Uuid,
    #[diesel(sql_type = Integer)]
    quantity: i32,
}

/// Cambia el estado de UNA línea y recalcula el del pedido, en una transacción.
///
/// La comprobación va en el propio `UPDATE` (línea, pedido, tienda del `scope`
/// y estado de partida): si no afecta a ninguna fila no se ha cambiado nada, y
/// dos personas pulsando a la vez no pueden aplicar el mismo cambio dos veces.
/// Eso es lo que impide devolver el stock dos veces al cancelar.
pub async fn set_line_status(
    scope: Option<&Uuid>,
    order_id: &Uuid,
    item_id: &Uuid,
    new_status: i16,
    pool: &DbPool,
) -> Result<(), PanelOrderError> {
    let from_status = required_status(new_status).ok_or(PanelOrderError::InvalidTransition)?;
    let mut conn = connection(pool).await?;

    conn.transaction::<(), PanelOrderError, _>(|conn| async move {
        let changed = sql_query(
            "UPDATE order_items \
             SET status = $1 \
             WHERE id = $2 AND order_id = $3 AND status = $4 \
               AND ($5::uuid IS NULL OR store_id = $5) \
             RETURNING product_var_id, quantity"
        )
            .bind::<SmallInt, _>(new_status)
            .bind::<SqlUuid, _>(item_id)
            .bind::<SqlUuid, _>(order_id)
            .bind::<SmallInt, _>(from_status)
            .bind::<Nullable<SqlUuid>, _>(scope)
            .get_result::<ChangedLine>(conn)
            .await
            .optional()?
            .ok_or(PanelOrderError::InvalidTransition)?;

        // Lo cancelado no se ha enviado: sus unidades vuelven a estar a la
        // venta. El importe del pedido no se toca (no hay pagos ni devoluciones).
        if new_status == ORDER_CANCELLED {
            sql_query("UPDATE product_variations SET stock = stock + $2 WHERE id = $1")
                .bind::<SqlUuid, _>(changed.product_var_id)
                .bind::<Integer, _>(changed.quantity)
                .execute(conn)
                .await?;
        }

        refresh_order_status(order_id, conn).await
    }.scope_boxed()).await
}

/// `orders.status` = el estado de la línea más atrasada de las no canceladas, o
/// cancelado si lo están todas. Misma regla que `summary_status`.
async fn refresh_order_status(order_id: &Uuid, conn: &mut AsyncPgConnection) -> Result<(), PanelOrderError> {
    sql_query(
        "UPDATE orders \
         SET status = COALESCE( \
             (SELECT MIN(status) FROM order_items WHERE order_id = $1 AND status <> $2), \
             $2) \
         WHERE id = $1"
    )
        .bind::<SqlUuid, _>(order_id)
        .bind::<SmallInt, _>(ORDER_CANCELLED)
        .execute(conn)
        .await?;
    Ok(())
}
