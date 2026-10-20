//! Cabeceras y respuestas de htmx.

use actix_web::http::StatusCode;
use actix_web::{HttpResponse, HttpResponseBuilder};
use askama::Template;
use serde_json::Value;

use crate::models::components::message_modal::MessageModal;

/// Respuesta de error (4xx/5xx) que htmx sí pinta: htmx 2 no pinta los
/// errores por defecto, pero con `HX-Retarget` el manejador `htmx:beforeSwap`
/// de `ui/utils/base.js` la deja pasar. Va al final de `<body>`, como los
/// avisos de las respuestas buenas. Para añadirle más cabeceras (HX-Trigger)
/// antes del cuerpo.
pub fn error_response(status: StatusCode) -> HttpResponseBuilder {
    let mut builder = HttpResponse::build(status);
    builder
        .insert_header(("HX-Retarget", "body"))
        .insert_header(("HX-Reswap", "beforeend"));
    builder
}

/// El aviso (MessageModal) con su código de error de verdad.
pub fn error_modal(status: StatusCode, text: &str) -> HttpResponse {
    error_response(status).body(MessageModal::new(text).render().unwrap())
}

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
    fn el_error_lleva_su_codigo_y_lo_que_necesita_htmx() {
        let resp = error_modal(StatusCode::CONFLICT, "Ese SKU ya existe");
        assert_eq!(resp.status(), StatusCode::CONFLICT);
        assert_eq!(resp.headers().get("HX-Retarget").unwrap(), "body");
        assert_eq!(resp.headers().get("HX-Reswap").unwrap(), "beforeend");
    }

    #[test]
    fn deja_la_cabecera_en_ascii_y_el_json_igual() {
        let value = json!({ "m": "Ya lo usa Cortina · Señal 😀" });
        let header = trigger_json(&value);
        assert!(header.is_ascii());
        assert_eq!(serde_json::from_str::<Value>(&header).unwrap(), value);
    }
}
