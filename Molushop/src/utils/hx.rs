//! Cabeceras de htmx.

use serde_json::Value;

/// El JSON de `HX-Trigger` solo con ASCII: una cabecera HTTP no admite tildes
/// ni `·` (actix respondería 500), y el texto del vendedor puede llevarlos.
/// Lo demás va como `\uXXXX`, que sigue siendo JSON y el navegador decodifica.
pub fn trigger_json(value: &Value) -> String {
    let mut out = String::new();
    for c in value.to_string().chars() {
        if c.is_ascii() {
            out.push(c);
        } else {
            let mut units = [0u16; 2];
            for unit in c.encode_utf16(&mut units) {
                out.push_str(&format!("\\u{:04x}", unit));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deja_la_cabecera_en_ascii_y_el_json_igual() {
        let value = json!({ "m": "Ya lo usa Cortina · Señal 😀" });
        let header = trigger_json(&value);
        assert!(header.is_ascii());
        assert_eq!(serde_json::from_str::<Value>(&header).unwrap(), value);
    }
}
