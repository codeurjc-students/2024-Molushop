//! IP del cliente, para el límite de peticiones y la sesión guardada.
//!
//! Detrás de un proxy (Render, Caddy, nginx...) la conexión llega desde el
//! proxy, así que `peer_addr` sería la misma IP para todo el mundo. El proxy
//! pone la IP real en una cabecera, y `CLIENT_IP_HEADER` dice cuál:
//!
//! - Render: `True-Client-IP` (la pone Cloudflare, que va delante de Render, y
//!   sustituye lo que mande el cliente). NO `X-Forwarded-For`: Render añade la
//!   IP real a lo que traiga la petición, y el cliente puede inventarse el resto.
//! - Caddy/nginx propios: hacer que el proxy ponga `X-Real-IP` con la IP de la
//!   conexión y usar `CLIENT_IP_HEADER=X-Real-IP`.
//! - Sin la variable (desarrollo): la IP de la conexión.
//!
//! Solo vale si nadie llega a la app sin pasar por el proxy: si no, cualquiera
//! podría mandar la cabecera con la IP que quisiera.

use std::env;
use std::net::{IpAddr, SocketAddr};

use actix_web::http::header::HeaderMap;
use actix_web::HttpRequest;
use lazy_static::lazy_static;

lazy_static! {
    static ref CLIENT_IP_HEADER: Option<String> = env::var("CLIENT_IP_HEADER")
        .ok()
        .map(|h| h.trim().to_string())
        .filter(|h| !h.is_empty());
}

/// La IP del cliente según `CLIENT_IP_HEADER` o, sin ella, la de la conexión.
pub fn client_ip(req: &HttpRequest) -> Option<IpAddr> {
    pick_ip(CLIENT_IP_HEADER.as_deref(), req.headers(), req.peer_addr())
}

/// Si la cabecera no viene o no es una IP, se usa la de la conexión: la petición
/// no ha pasado por el proxy (o el proxy está mal configurado).
fn pick_ip(header: Option<&str>, headers: &HeaderMap, peer: Option<SocketAddr>) -> Option<IpAddr> {
    header
        .and_then(|name| headers.get(name))
        .and_then(|value| value.to_str().ok())
        // Por si el proxy pone una lista: la primera es la del cliente.
        .and_then(|value| value.split(',').next())
        .and_then(|ip| ip.trim().parse().ok())
        .or_else(|| peer.map(|addr| addr.ip()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::header::{HeaderName, HeaderValue};

    fn headers(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.insert(HeaderName::from_static(name), HeaderValue::from_static(value));
        }
        map
    }

    fn peer() -> Option<SocketAddr> {
        Some("10.0.0.5:40000".parse().unwrap())
    }

    #[test]
    fn without_config_uses_the_connection_ip() {
        let h = headers(&[("true-client-ip", "203.0.113.7")]);
        assert_eq!(pick_ip(None, &h, peer()), Some("10.0.0.5".parse().unwrap()));
    }

    #[test]
    fn with_config_uses_the_header() {
        let h = headers(&[("true-client-ip", "203.0.113.7")]);
        assert_eq!(pick_ip(Some("True-Client-IP"), &h, peer()), Some("203.0.113.7".parse().unwrap()));
    }

    #[test]
    fn header_list_takes_the_first_ip() {
        let h = headers(&[("x-real-ip", " 2001:db8::1 , 10.0.0.1")]);
        assert_eq!(pick_ip(Some("X-Real-IP"), &h, peer()), Some("2001:db8::1".parse().unwrap()));
    }

    #[test]
    fn missing_or_invalid_header_falls_back_to_the_connection() {
        assert_eq!(pick_ip(Some("True-Client-IP"), &headers(&[]), peer()), Some("10.0.0.5".parse().unwrap()));
        let h = headers(&[("true-client-ip", "no-es-una-ip")]);
        assert_eq!(pick_ip(Some("True-Client-IP"), &h, peer()), Some("10.0.0.5".parse().unwrap()));
    }
}
