//! SKU de las variaciones: el código interno que pone cada tienda. Opcional y
//! único dentro de la tienda (índice `product_variations_store_sku`; dos
//! tiendas pueden repetirlo).

use diesel::result::{DatabaseErrorKind, Error};
use diesel::sql_query;
use diesel::OptionalExtension;
use diesel::sql_types::{Nullable, SmallInt, Text, Uuid as SqlUuid};
use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

type DbPool = Pool<AsyncPgConnection>;

pub const SKU_MAX_LEN: usize = 64;

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
}
