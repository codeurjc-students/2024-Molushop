//! Segunda barrera contra CSRF (la primera es la cookie `SameSite=Lax`): una
//! petición que cambia algo (POST, PUT, PATCH, DELETE) tiene que venir de esta
//! misma web. El navegador pone siempre `Origin` en esas peticiones, y una web
//! ajena no lo puede falsificar.
//!
//! - `Origin` presente: su host:puerto tiene que ser el de la petición (`Host`)
//!   o uno de `ALLOWED_ORIGINS` (lista separada por comas en el `.env`, para
//!   cuando haya un dominio o un proxy delante).
//! - Sin `Origin` (navegadores muy viejos, `curl`, herramientas): se mira
//!   `Referer` igual; sin ninguno de los dos se deja pasar, porque no viene de
//!   un navegador que lleve la cookie de otra web.
//!
//! Se pone una vez para toda la app en `main.rs`: `.wrap(from_fn(origin_check))`.

use std::env;

use actix_web::body::{BoxBody, MessageBody};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::{header, Method};
use actix_web::middleware::Next;
use actix_web::{Error, HttpResponse};

/// "http://127.0.0.1:8080/ruta" -> "127.0.0.1:8080". `None` si no es una URL
/// http(s) (p. ej. el `Origin: null` de un iframe sin origen).
fn host_of(url: &str) -> Option<&str> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let host = rest.split('/').next()?;
    if host.is_empty() { None } else { Some(host) }
}

/// ¿Viene `source` (un `Origin` o un `Referer`) de `host` o de uno permitido?
fn same_site(source: &str, host: &str, allowed: &[String]) -> bool {
    match host_of(source) {
        Some(h) => h.eq_ignore_ascii_case(host) || allowed.iter().any(|a| host_of(a).is_some_and(|ah| h.eq_ignore_ascii_case(ah))),
        None => false,
    }
}

fn allowed_origins() -> Vec<String> {
    env::var("ALLOWED_ORIGINS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub async fn origin_check(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    let unsafe_method = matches!(*req.method(), Method::POST | Method::PUT | Method::PATCH | Method::DELETE);
    if unsafe_method {
        let source = req
            .headers()
            .get(header::ORIGIN)
            .or_else(|| req.headers().get(header::REFERER))
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        if let Some(source) = source {
            let host = req.connection_info().host().to_string();
            if !same_site(&source, &host, &allowed_origins()) {
                println!("Origin rechazado: {} {} desde {}", req.method(), req.path(), source);
                let res = HttpResponse::Forbidden()
                    .content_type("text/html; charset=utf-8")
                    .body("<h1>403</h1><p>Petición rechazada: no viene de esta web.</p>");
                return Ok(req.into_response(res));
            }
        }
    }
    Ok(next.call(req).await?.map_into_boxed_body())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_host_passes() {
        assert!(same_site("http://127.0.0.1:8080", "127.0.0.1:8080", &[]));
        assert!(same_site("http://127.0.0.1:8080/cart", "127.0.0.1:8080", &[]));
        assert!(same_site("https://Molushop.com", "molushop.com", &[]));
    }

    #[test]
    fn other_host_or_port_fails() {
        assert!(!same_site("https://evil.example", "127.0.0.1:8080", &[]));
        assert!(!same_site("http://127.0.0.1:9999", "127.0.0.1:8080", &[]));
        // Un host que solo empieza igual no cuela.
        assert!(!same_site("http://127.0.0.1:8080.evil.example", "127.0.0.1:8080", &[]));
        assert!(!same_site("null", "127.0.0.1:8080", &[]));
    }

    #[test]
    fn allowed_origins_pass() {
        let allowed = vec!["https://www.molushop.com".to_string()];
        assert!(same_site("https://www.molushop.com", "10.0.0.5:8080", &allowed));
        assert!(!same_site("https://otra.com", "10.0.0.5:8080", &allowed));
    }
}
