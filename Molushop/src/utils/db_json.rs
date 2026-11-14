//! Columnas JSON de la BD (atributos de una variación, imágenes, títulos de
//! variación...) convertidas a structs de Rust.
//!
//! Si el JSON guardado no encaja con el struct (un dato viejo, uno escrito a
//! mano), la página tiene que salir igual: se usa `None` (y el que llama pone
//! su valor por defecto) y queda un `warn!` en el log con qué campo y de qué
//! fila. Antes eran `.unwrap()`: un solo dato raro cortaba la petición entera
//! sin respuesta para el usuario.

use serde::de::DeserializeOwned;
use serde_json::{from_value, Value};
use uuid::Uuid;

/// `None` si la columna es NULL o si su JSON no encaja con `T` (esto último,
/// con aviso en el log). `field` es el nombre de la columna e `id`, el de la
/// fila, para poder encontrar el dato malo.
pub fn parse_db_json<T: DeserializeOwned>(value: Option<Value>, field: &'static str, id: &Uuid) -> Option<T> {
    match from_value::<T>(value?) {
        Ok(parsed) => Some(parsed),
        Err(e) => {
            tracing::warn!(%id, field, error = %e, "could not parse a JSON column");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn null_column_is_none() {
        assert_eq!(parse_db_json::<Vec<String>>(None, "attributes", &Uuid::nil()), None);
    }

    #[test]
    fn valid_json_is_parsed() {
        let parsed = parse_db_json::<Vec<String>>(Some(json!(["a", "b"])), "attributes", &Uuid::nil());
        assert_eq!(parsed, Some(vec!["a".to_string(), "b".to_string()]));
    }

    /// Un JSON que no encaja ya no hace panic: da `None` (y avisa en el log).
    #[test]
    fn mismatched_json_is_none_instead_of_panicking() {
        let parsed = parse_db_json::<Vec<String>>(Some(json!({"not": "a list"})), "attributes", &Uuid::nil());
        assert_eq!(parsed, None);
    }
}
