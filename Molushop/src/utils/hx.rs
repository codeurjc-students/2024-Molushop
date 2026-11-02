//! Cabeceras y respuestas de htmx.

use actix_web::http::StatusCode;
use actix_web::{HttpResponse, HttpResponseBuilder};
use askama::Template;
use serde_json::Value;

use crate::models::components::message_modal::MessageModal;
use crate::i18n::I18n;

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

/// El aviso (MessageModal) con su código de error de verdad y la referencia
/// de la petición ("Error 500 · ref. 7f3a9c2e").
pub fn error_modal(status: StatusCode, text: &str) -> HttpResponse {
    let modal = MessageModal::new(text).with_error_ref(status, I18n::default_ref());
    error_response(status).body(modal.render().unwrap())
}

/// Como `error_modal`, en el idioma de la página: `key` es una clave de
/// `locales/*` y el botón va traducido.
pub fn error_modal_t(status: StatusCode, i18n: &I18n, key: &str) -> HttpResponse {
    let modal = MessageModal::translated(i18n, key).with_error_ref(status, i18n);
    error_response(status).body(modal.render().unwrap())
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

    /// El aviso lleva su código de error de verdad y las cabeceras para que htmx lo pinte en el body.
    #[test]
    fn error_has_its_status_and_htmx_headers() {
        let resp = error_modal(StatusCode::CONFLICT, "Ese SKU ya existe");
        assert_eq!(resp.status(), StatusCode::CONFLICT);
        assert_eq!(resp.headers().get("HX-Retarget").unwrap(), "body");
        assert_eq!(resp.headers().get("HX-Reswap").unwrap(), "beforeend");
    }

    /// Tildes y emojis van escapados: una cabecera HTTP solo admite ASCII, y el JSON decodificado no cambia.
    #[test]
    fn header_is_ascii_and_json_unchanged() {
        let value = json!({ "m": "Ya lo usa Cortina · Señal 😀" });
        let header = trigger_json(&value);
        assert!(header.is_ascii());
        assert_eq!(serde_json::from_str::<Value>(&header).unwrap(), value);
    }
}
