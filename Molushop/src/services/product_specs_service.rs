//! Especificaciones de un producto en el panel (pestaña de "Editar producto").
//! Se guardan en `products.specs` como un objeto `{nombre: valor}`; la
//! categoría propone algunos nombres (`category.base_specs`), como en el alta.

use diesel::result::Error;
use diesel::sql_query;
use diesel::sql_types::{Jsonb, Nullable, Uuid as SqlUuid};
use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::RunQueryDsl;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::models::components::edit_product::Specs;
use crate::models::error::ServiceError;

type DbPool = Pool<AsyncPgConnection>;

#[derive(diesel::QueryableByName)]
struct SpecsRow {
    #[diesel(sql_type = Nullable<Jsonb>)]
    specs: Option<Value>,
    #[diesel(sql_type = Nullable<Jsonb>)]
    base_specs: Option<Value>,
}

/// Las del producto, separadas en las que propone su categoría (en el orden
/// de la categoría) y las demás (por nombre).
pub async fn get_specs(product_id: &Uuid, pool: &DbPool) -> Result<Specs, Error> {
    let mut conn = pool.get().await.unwrap();
    let row = sql_query(
        "SELECT p.specs,
                (SELECT c.base_specs FROM category_product cp
                 JOIN category c ON c.id = cp.category_id
                 WHERE cp.product_id = p.id LIMIT 1) AS base_specs
         FROM products p WHERE p.id = $1"
    )
        .bind::<SqlUuid, _>(product_id)
        .get_result::<SpecsRow>(&mut conn)
        .await?;
    Ok(split_specs(row.specs.as_ref(), row.base_specs.as_ref()))
}

/// `base_specs` es `{"specs": ["Peso", ...]}` (como lo lee el alta); si no
/// encaja, no hay sugeridas. Un valor que no es texto se enseña como JSON.
pub fn split_specs(specs: Option<&Value>, base_specs: Option<&Value>) -> Specs {
    let saved = specs.and_then(Value::as_object);
    let value_of = |key: &str| -> String {
        match saved.and_then(|map| map.get(key)) {
            Some(Value::String(s)) => s.clone(),
            Some(other) => other.to_string(),
            None => String::new(),
        }
    };
    let suggested: Vec<String> = base_specs
        .and_then(|b| b.get("specs"))
        .and_then(Value::as_array)
        .map(|names| names.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default();

    let category = suggested.iter().map(|key| (key.clone(), value_of(key))).collect();
    let mut custom: Vec<(String, String)> = saved
        .map(|map| map.keys()
            .filter(|key| !suggested.contains(key))
            .map(|key| (key.clone(), value_of(key)))
            .collect())
        .unwrap_or_default();
    custom.sort();
    Specs { category, custom }
}

/// Sustituye las specs del producto por las del formulario, limpias como en el
/// alta (`clean_specs`: sin nombre o sin valor no se guardan). Quien llama ya
/// ha comprobado que el producto es del usuario.
pub async fn save_specs(product_id: &Uuid, raw: &Value, pool: &DbPool) -> Result<(), ServiceError> {
    let cleaned = clean_specs(raw)?;
    let mut conn = pool.get().await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    sql_query("UPDATE products SET specs = $2 WHERE id = $1")
        .bind::<SqlUuid, _>(product_id)
        .bind::<Jsonb, _>(&cleaned)
        .execute(&mut conn)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;
    Ok(())
}

pub fn clean_specs(raw: &Value) -> Result<Value, ServiceError> {
    let map = match raw {
        Value::Null => return Ok(json!({})),
        Value::Object(map) => map,
        _ => return Err(ServiceError::InvalidSpecs),
    };
    let mut cleaned = serde_json::Map::new();
    for (key, value) in map {
        let value = value.as_str().ok_or(ServiceError::InvalidSpecs)?.trim();
        let key = key.trim();
        if !key.is_empty() && !value.is_empty() {
            cleaned.insert(key.to_string(), Value::String(value.to_string()));
        }
    }
    Ok(Value::Object(cleaned))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Separa las specs de la plantilla de la categoría (en su orden, vacías si faltan) de las que añadió el vendedor.
    #[test]
    fn splits_category_specs_from_own() {
        let specs = json!({ "os": "Android", "zz": "1", "peso": 200 });
        let base = json!({ "specs": ["screen_size", "os"] });
        let split = split_specs(Some(&specs), Some(&base));
        assert_eq!(split.category, vec![
            ("screen_size".to_string(), String::new()),
            ("os".to_string(), "Android".to_string()),
        ]);
        assert_eq!(split.custom, vec![
            ("peso".to_string(), "200".to_string()),
            ("zz".to_string(), "1".to_string()),
        ]);
    }

    /// Sin specs y con una plantilla que no se entiende, no hay nada que pintar.
    #[test]
    fn no_category_and_no_specs_is_empty() {
        let split = split_specs(None, Some(&json!("no vale")));
        assert!(split.category.is_empty() && split.custom.is_empty());
    }

    /// Quita las specs con el nombre o el valor en blanco y recorta los espacios.
    #[test]
    fn clean_specs_drops_empty_ones() {
        let raw = json!({"material": " algodón ", "size": "", "": "suelto", "  ": "x"});
        assert_eq!(clean_specs(&raw).unwrap(), json!({"material": "algodón"}));
    }

    /// Sin specs (null o todas en blanco) queda un objeto vacío, no null.
    #[test]
    fn clean_specs_without_specs_is_empty_object() {
        assert_eq!(clean_specs(&Value::Null).unwrap(), json!({}));
        assert_eq!(clean_specs(&json!({"size": "  "})).unwrap(), json!({}));
    }

    /// Rechaza lo que no sea un objeto de textos nombre → valor.
    #[test]
    fn clean_specs_rejects_non_name_value() {
        assert!(matches!(clean_specs(&json!(["a"])), Err(ServiceError::InvalidSpecs)));
        assert!(matches!(clean_specs(&json!({"size": 3})), Err(ServiceError::InvalidSpecs)));
    }
}
