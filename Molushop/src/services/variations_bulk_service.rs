//! Edición en bloque de las variaciones de un producto (pestaña Variaciones de
//! "Editar producto"): el vendedor filtra en el explorador y aplica una acción
//! a todas las que quedan a la vista.

use bigdecimal::{BigDecimal, RoundingMode};
use diesel::result::Error;
use diesel::sql_query;
use diesel::sql_types::{Array, BigInt, Integer, Nullable, Numeric, SmallInt, Text, Uuid as SqlUuid};
use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use serde::Serialize;
use std::str::FromStr;
use uuid::Uuid;

use crate::constants::product_status::{PRODUCT_ACTIVE, PRODUCT_DRAFT, PRODUCT_INACTIVE};

type DbPool = Pool<AsyncPgConnection>;

/// La moneda de los precios del panel (como en `edit_price`).
const CURRENCY: &str = "EUR";

#[derive(Debug, Clone, PartialEq)]
pub enum BulkAction {
    Status(i16),
    SetStock(i32),
    /// Suma (o resta, si es negativo) unidades; el stock no baja de 0.
    AddStock(i32),
    Price(BigDecimal),
}

impl BulkAction {
    /// Lo que manda el formulario de la barra: `action` y su `value`. `Err`
    /// con el texto que se le enseña al vendedor.
    pub fn parse(action: &str, value: &str) -> Result<Self, &'static str> {
        let value = value.trim();
        match action {
            "status" => match value.parse::<i16>() {
                Ok(s) if [PRODUCT_ACTIVE, PRODUCT_INACTIVE, PRODUCT_DRAFT].contains(&s) => Ok(BulkAction::Status(s)),
                _ => Err("Elige un estado válido"),
            },
            "set_stock" => match value.parse::<i32>() {
                Ok(n) if n >= 0 => Ok(BulkAction::SetStock(n)),
                _ => Err("El stock debe ser un número entero, 0 o más"),
            },
            "add_stock" => match value.parse::<i32>() {
                Ok(n) if n != 0 => Ok(BulkAction::AddStock(n)),
                _ => Err("Indica cuántas unidades sumar (o restar, con un número negativo)"),
            },
            "price" => match BigDecimal::from_str(value) {
                // A céntimos, redondeando la mitad hacia arriba (12,345 -> 12,35)
                Ok(p) if p >= BigDecimal::from(0) => Ok(BulkAction::Price(p.with_scale_round(2, RoundingMode::HalfUp))),
                _ => Err("El precio debe ser un número, 0 o más"),
            },
            _ => Err("Acción no válida"),
        }
    }
}

/// Cómo queda cada variación tras el cambio: el JS del explorador lo copia en
/// su fila (`variations_bulk_update`).
#[derive(diesel::QueryableByName, Serialize, Debug)]
pub struct BulkRow {
    #[diesel(sql_type = SqlUuid)]
    pub id: Uuid,
    #[diesel(sql_type = SmallInt)]
    pub status: i16,
    #[diesel(sql_type = Integer)]
    pub stock: i32,
    #[diesel(sql_type = Nullable<Text>)]
    pub price: Option<String>,
}

#[derive(diesel::QueryableByName)]
struct Count {
    #[diesel(sql_type = BigInt)]
    n: i64,
}

#[derive(Debug)]
pub enum BulkOutcome {
    /// Cómo han quedado todas.
    Updated(Vec<BulkRow>),
    /// Alguna no es del producto: no se ha cambiado nada.
    NotOwned,
    /// Se querían activar y estas no tienen precio (no se ha cambiado nada):
    /// saldrían a la venta sin poder comprarse.
    MissingPrice(i64),
}

/// Aplica `action` a las variaciones `ids` del producto, en una transacción.
/// `NotOwned` (sin cambiar nada) si alguna no es de ese producto: quien llama
/// ya ha comprobado que el producto es del usuario (`product_access`), y así
/// no se cuela una variación de otro.
pub async fn bulk_update_variations(
    product_id: &Uuid,
    ids: &[Uuid],
    action: &BulkAction,
    pool: &DbPool,
) -> Result<BulkOutcome, Error> {
    let mut ids = ids.to_vec();
    ids.sort();
    ids.dedup();
    let product_id = *product_id;
    let action = action.clone();
    let mut conn = pool.get().await.unwrap();

    conn.transaction::<BulkOutcome, Error, _>(|conn| async move {
        let owned = sql_query(
            "SELECT count(*) AS n FROM product_variations WHERE product_id = $1 AND id = ANY($2)"
        )
            .bind::<SqlUuid, _>(product_id)
            .bind::<Array<SqlUuid>, _>(&ids)
            .get_result::<Count>(conn)
            .await?;
        if ids.is_empty() || owned.n != ids.len() as i64 {
            return Ok(BulkOutcome::NotOwned);
        }

        // Activar: ninguna que pase a activa puede quedarse sin precio (las que
        // ya lo estaban, de antes de esta regla, no cuentan).
        if action == BulkAction::Status(PRODUCT_ACTIVE) {
            let missing = sql_query(
                "SELECT count(*) AS n FROM product_variations v
                 WHERE v.id = ANY($1) AND v.status <> $2
                   AND NOT EXISTS (SELECT 1 FROM prices pr WHERE pr.variation_id = v.id AND pr.currency = $3)"
            )
                .bind::<Array<SqlUuid>, _>(&ids)
                .bind::<SmallInt, _>(PRODUCT_ACTIVE)
                .bind::<Text, _>(CURRENCY)
                .get_result::<Count>(conn)
                .await?;
            if missing.n > 0 {
                return Ok(BulkOutcome::MissingPrice(missing.n));
            }
        }

        match &action {
            BulkAction::Status(status) => {
                sql_query("UPDATE product_variations SET status = $2 WHERE id = ANY($1)")
                    .bind::<Array<SqlUuid>, _>(&ids)
                    .bind::<SmallInt, _>(*status)
                    .execute(conn).await?;
            }
            BulkAction::SetStock(stock) => {
                sql_query("UPDATE product_variations SET stock = $2 WHERE id = ANY($1)")
                    .bind::<Array<SqlUuid>, _>(&ids)
                    .bind::<Integer, _>(*stock)
                    .execute(conn).await?;
            }
            BulkAction::AddStock(delta) => {
                sql_query("UPDATE product_variations SET stock = GREATEST(stock + $2, 0) WHERE id = ANY($1)")
                    .bind::<Array<SqlUuid>, _>(&ids)
                    .bind::<Integer, _>(*delta)
                    .execute(conn).await?;
            }
            BulkAction::Price(price) => {
                // Como `update_price_variation`: crea el precio si falta y el
                // trigger `price_history_trigger` lleva el historial. El WHERE
                // deja sin tocar las que ya tienen ese precio, para no abrir
                // en el historial filas que no cambian nada.
                sql_query(
                    "INSERT INTO prices (variation_id, price, currency, start_date)
                     SELECT v, $2, $3, (now() AT TIME ZONE 'UTC') FROM unnest($1::uuid[]) AS v
                     ON CONFLICT (variation_id, currency) DO UPDATE
                        SET price = EXCLUDED.price, start_date = EXCLUDED.start_date
                        WHERE prices.price IS DISTINCT FROM EXCLUDED.price"
                )
                    .bind::<Array<SqlUuid>, _>(&ids)
                    .bind::<Numeric, _>(price)
                    .bind::<Text, _>(CURRENCY)
                    .execute(conn).await?;
            }
        }

        let rows = sql_query(
            "SELECT pv.id, pv.status, pv.stock, pr.price::text AS price
             FROM product_variations pv
             LEFT JOIN prices pr ON pr.variation_id = pv.id AND pr.currency = $2
             WHERE pv.id = ANY($1)"
        )
            .bind::<Array<SqlUuid>, _>(&ids)
            .bind::<Text, _>(CURRENCY)
            .load::<BulkRow>(conn)
            .await?;
        Ok(BulkOutcome::Updated(rows))
    }.scope_boxed()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valida_cada_accion() {
        assert_eq!(BulkAction::parse("status", "1"), Ok(BulkAction::Status(PRODUCT_ACTIVE)));
        assert!(BulkAction::parse("status", "3").is_err(), "3 es borrado: no se pone desde aquí");
        assert_eq!(BulkAction::parse("set_stock", " 10 "), Ok(BulkAction::SetStock(10)));
        assert!(BulkAction::parse("set_stock", "-1").is_err());
        assert!(BulkAction::parse("set_stock", "2.5").is_err());
        assert_eq!(BulkAction::parse("add_stock", "-3"), Ok(BulkAction::AddStock(-3)));
        assert!(BulkAction::parse("add_stock", "0").is_err());
        assert_eq!(BulkAction::parse("price", "12.345"), Ok(BulkAction::Price(BigDecimal::from_str("12.35").unwrap())));
        assert!(BulkAction::parse("price", "-1").is_err());
        assert!(BulkAction::parse("price", "abc").is_err());
        assert!(BulkAction::parse("delete", "1").is_err());
    }
}
