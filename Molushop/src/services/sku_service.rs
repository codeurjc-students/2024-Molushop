//! SKU de las variaciones: el código interno que pone cada tienda. Opcional y
//! único dentro de la tienda (índice `product_variations_store_sku`; dos
//! tiendas pueden repetirlo).

use diesel::result::{DatabaseErrorKind, Error};
use diesel::sql_query;
use diesel::OptionalExtension;
use diesel::sql_types::{Array, BigInt, Nullable, SmallInt, Text, Uuid as SqlUuid};
use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use serde::Serialize;
use crate::constants::product_status::PRODUCT_ACTIVE;
use std::collections::HashMap;
use uuid::Uuid;

type DbPool = Pool<AsyncPgConnection>;

pub const SKU_MAX_LEN: usize = 64;

/// La moneda de los precios del panel (como en `edit_price`).
const PANEL_CURRENCY: &str = "EUR";

/// Lo que escribe el vendedor, listo para guardar: sin espacios en los lados,
/// `None` si queda vacío. `Err` con el texto que se le enseña.
pub fn normalize_sku(raw: &str) -> Result<Option<String>, &'static str> {
    let sku = raw.trim();
    if sku.is_empty() {
        return Ok(None);
    }
    if sku.chars().count() > SKU_MAX_LEN {
        return Err("El SKU no puede pasar de 64 caracteres");
    }
    if !sku.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')) {
        return Err("El SKU solo puede llevar letras sin tilde, números, guiones (-), guiones bajos (_) y puntos (.)");
    }
    Ok(Some(sku.to_string()))
}

/// Resultado de guardar el General de una variación.
#[derive(Debug)]
pub enum SaveGeneral {
    Saved,
    /// El SKU ya lo tiene otra variación de la tienda: cuál ("Cortina · Verde · S").
    SkuTaken(String),
    /// Se quería activar y no tiene precio: saldría a la venta sin poder comprarse.
    NoPrice,
}

#[derive(diesel::QueryableByName)]
struct Blocked {
    #[diesel(sql_type = diesel::sql_types::Bool)]
    blocked: bool,
}

#[derive(diesel::QueryableByName)]
struct Holder {
    #[diesel(sql_type = Text)]
    label: String,
}

/// Estado y SKU de una variación, juntos: si el SKU está cogido no se guarda
/// nada. Quien llama ya ha comprobado que la variación es del usuario.
pub async fn save_variation_general(
    variation_id: &Uuid,
    status: i16,
    sku: Option<String>,
    pool: &DbPool,
) -> Result<SaveGeneral, Error> {
    let variation_id = *variation_id;
    let mut conn = pool.get().await.unwrap();

    let result = conn.transaction::<SaveGeneral, Error, _>(|conn| async move {
        // Solo al pasar a activa: una que ya lo estaba sin precio (de antes de
        // esta regla) se puede seguir editando.
        if status == PRODUCT_ACTIVE {
            let check = sql_query(
                "SELECT (v.status <> $2 AND NOT EXISTS (
                        SELECT 1 FROM prices pr WHERE pr.variation_id = v.id AND pr.currency = $3)) AS blocked
                 FROM product_variations v WHERE v.id = $1"
            )
                .bind::<SqlUuid, _>(variation_id)
                .bind::<SmallInt, _>(PRODUCT_ACTIVE)
                .bind::<Text, _>(PANEL_CURRENCY)
                .get_result::<Blocked>(conn)
                .await?;
            if check.blocked {
                return Ok(SaveGeneral::NoPrice);
            }
        }
        if let Some(sku) = &sku {
            let holder = sql_query(
                "SELECT p.name || COALESCE(' · ' || (
                        SELECT string_agg(attr->>'value', ' · ')
                        FROM jsonb_array_elements(v.attributes) AS attr), '') AS label
                 FROM product_variations v
                 JOIN products p ON p.id = v.product_id
                 WHERE v.sku = $2 AND v.id <> $1
                   AND v.store_id = (SELECT store_id FROM product_variations WHERE id = $1)
                 LIMIT 1"
            )
                .bind::<SqlUuid, _>(variation_id)
                .bind::<Text, _>(sku)
                .get_result::<Holder>(conn)
                .await
                .optional()?;
            if let Some(holder) = holder {
                return Ok(SaveGeneral::SkuTaken(holder.label));
            }
        }
        sql_query("UPDATE product_variations SET status = $2, sku = $3 WHERE id = $1")
            .bind::<SqlUuid, _>(variation_id)
            .bind::<SmallInt, _>(status)
            .bind::<Nullable<Text>, _>(sku)
            .execute(conn)
            .await?;
        Ok(SaveGeneral::Saved)
    }.scope_boxed()).await;

    // Dos guardados a la vez pueden pasar los dos la comprobación: el índice
    // único para al segundo.
    match result {
        Err(Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
            Ok(SaveGeneral::SkuTaken("otra variación de la tienda".to_string()))
        }
        other => other,
    }
}

/// Un SKU del lote que no se puede guardar: de qué variación y por qué.
#[derive(Debug, Serialize, PartialEq)]
pub struct SkuProblem {
    pub id: Uuid,
    pub message: String,
}

/// Cómo queda una variación: el JS del explorador lo copia en su fila.
#[derive(Debug, Serialize)]
pub struct SkuRow {
    pub id: Uuid,
    /// Vacío si se queda sin SKU.
    pub sku: String,
}

#[derive(Debug)]
pub enum SaveSkus {
    Saved(Vec<SkuRow>),
    /// Alguna variación no es del producto: no se ha cambiado nada.
    NotOwned,
    /// Ninguno guardado. Vacío: otro guardado a la vez cogió alguno (lo paró el
    /// índice único) y no se sabe cuál.
    Problems(Vec<SkuProblem>),
}

/// Lo que se comprueba sin la base de datos: el formato de cada uno y que no
/// se repitan dentro del lote. `Ok` con los SKU ya normalizados.
pub fn check_sku_batch(changes: &[(Uuid, String)]) -> Result<Vec<(Uuid, Option<String>)>, Vec<SkuProblem>> {
    let mut problems = Vec::new();
    let mut checked = Vec::new();
    for (id, raw) in changes {
        match normalize_sku(raw) {
            Ok(sku) => checked.push((*id, sku)),
            Err(message) => problems.push(SkuProblem { id: *id, message: message.to_string() }),
        }
    }
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for (_, sku) in &checked {
        if let Some(sku) = sku {
            *seen.entry(sku.as_str()).or_default() += 1;
        }
    }
    for (id, sku) in &checked {
        if let Some(sku) = sku {
            if seen[sku.as_str()] > 1 {
                problems.push(SkuProblem { id: *id, message: format!("{} está repetido en estos cambios", sku) });
            }
        }
    }
    if problems.is_empty() { Ok(checked) } else { Err(problems) }
}

#[derive(diesel::QueryableByName)]
struct Count {
    #[diesel(sql_type = BigInt)]
    n: i64,
}

#[derive(diesel::QueryableByName)]
struct Taken {
    #[diesel(sql_type = Text)]
    sku: String,
    #[diesel(sql_type = Text)]
    label: String,
}

/// Los SKU de varias variaciones del producto a la vez (tabla de Editar
/// producto), todos o ninguno. Quien llama ya ha comprobado que el producto es
/// del usuario (`product_access`).
pub async fn save_skus(
    product_id: &Uuid,
    changes: &[(Uuid, String)],
    pool: &DbPool,
) -> Result<SaveSkus, Error> {
    let checked = match check_sku_batch(changes) {
        Ok(checked) => checked,
        Err(problems) => return Ok(SaveSkus::Problems(problems)),
    };
    let ids: Vec<Uuid> = checked.iter().map(|(id, _)| *id).collect();
    let skus: Vec<Option<String>> = checked.iter().map(|(_, sku)| sku.clone()).collect();
    let product_id = *product_id;
    let mut conn = pool.get().await.unwrap();

    let result = conn.transaction::<SaveSkus, Error, _>(|conn| async move {
        let owned = sql_query(
            "SELECT count(DISTINCT id) AS n FROM product_variations WHERE product_id = $1 AND id = ANY($2)"
        )
            .bind::<SqlUuid, _>(product_id)
            .bind::<Array<SqlUuid>, _>(&ids)
            .get_result::<Count>(conn)
            .await?;
        if ids.is_empty() || owned.n != ids.len() as i64 {
            return Ok(SaveSkus::NotOwned);
        }

        // Los que ya tiene otra variación de la tienda que no está en el lote
        // (las del lote pueden intercambiarse los SKU entre ellas).
        let taken = sql_query(
            "SELECT v.sku, p.name || COALESCE(' · ' || (
                    SELECT string_agg(attr->>'value', ' · ')
                    FROM jsonb_array_elements(v.attributes) AS attr), '') AS label
             FROM product_variations v
             JOIN products p ON p.id = v.product_id
             WHERE v.sku = ANY($3) AND v.id <> ALL($2)
               AND v.store_id = (SELECT store_id FROM product_variations WHERE product_id = $1 LIMIT 1)"
        )
            .bind::<SqlUuid, _>(product_id)
            .bind::<Array<SqlUuid>, _>(&ids)
            .bind::<Array<Nullable<Text>>, _>(&skus)
            .load::<Taken>(conn)
            .await?;
        if !taken.is_empty() {
            let problems = checked.iter()
                .filter_map(|(id, sku)| {
                    let holder = taken.iter().find(|t| Some(&t.sku) == sku.as_ref())?;
                    Some(SkuProblem { id: *id, message: format!("Ya lo usa {}", holder.label) })
                })
                .collect();
            return Ok(SaveSkus::Problems(problems));
        }

        // Primero a NULL: si dos del lote se intercambian el SKU, el índice
        // no ve un duplicado a medias.
        sql_query("UPDATE product_variations SET sku = NULL WHERE id = ANY($1)")
            .bind::<Array<SqlUuid>, _>(&ids)
            .execute(conn).await?;
        sql_query(
            "UPDATE product_variations v SET sku = c.sku
             FROM unnest($1::uuid[], $2::text[]) AS c(id, sku)
             WHERE v.id = c.id"
        )
            .bind::<Array<SqlUuid>, _>(&ids)
            .bind::<Array<Nullable<Text>>, _>(&skus)
            .execute(conn).await?;

        let rows = checked.into_iter()
            .map(|(id, sku)| SkuRow { id, sku: sku.unwrap_or_default() })
            .collect();
        Ok(SaveSkus::Saved(rows))
    }.scope_boxed()).await;

    match result {
        Err(Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => Ok(SaveSkus::Problems(Vec::new())),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normaliza_el_sku() {
        assert_eq!(normalize_sku("  CORT-VER_S.1 "), Ok(Some("CORT-VER_S.1".to_string())));
        assert_eq!(normalize_sku("   "), Ok(None));
        assert_eq!(normalize_sku(""), Ok(None));
        assert!(normalize_sku("CORT VER").is_err(), "sin espacios en medio");
        assert!(normalize_sku("CAÑA").is_err(), "sin eñes ni tildes");
        assert!(normalize_sku(&"A".repeat(64)).is_ok());
        assert!(normalize_sku(&"A".repeat(65)).is_err());
    }

    #[test]
    fn el_lote_no_repite_ni_deja_formatos_malos() {
        let (a, b, c) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(
            check_sku_batch(&[(a, " X-1 ".into()), (b, "".into()), (c, "".into())]),
            Ok(vec![(a, Some("X-1".into())), (b, None), (c, None)]),
            "varias sin SKU no se repiten"
        );
        let problems = check_sku_batch(&[(a, "X-1".into()), (b, "X-1".into()), (c, "MAL SKU".into())]).unwrap_err();
        assert_eq!(problems.iter().map(|p| p.id).collect::<Vec<_>>(), vec![c, a, b]);
    }
}
