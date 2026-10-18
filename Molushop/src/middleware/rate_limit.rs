//! Límite de peticiones por IP para lo que se puede usar para adivinar
//! contraseñas o para crear cuentas en masa: login, registros, "olvidé mi
//! contraseña" y cambiar la contraseña.
//!
//! Ventana fija: como mucho `max` peticiones por IP cada `window`. Vive en
//! memoria del proceso (se reinicia con el servidor) y es compartido por todos
//! los workers (`lazy_static`). La IP es la de la conexión (`peer_addr`): detrás
//! de un proxy inverso habría que leer la del proxy de confianza.
//!
//! Se pone por ruta con `from_fn`, solo en el POST:
//!
//!     web::post().to(handler).wrap(from_fn(limit_panel_register))
//!     #[post("/login", wrap = "from_fn(crate::middleware::rate_limit::limit_login)")]

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use actix_web::body::{BoxBody, MessageBody};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header;
use actix_web::middleware::Next;
use actix_web::{Error, HttpResponse};
use askama::Template;
use lazy_static::lazy_static;

use crate::models::components::message_modal::MessageModal;

pub struct RateLimiter {
    max: u32,
    window: Duration,
    /// IP -> (inicio de su ventana, peticiones en ella).
    hits: Mutex<HashMap<IpAddr, (Instant, u32)>>,
}

/// A partir de cuántas IPs distintas se limpian las ventanas ya pasadas, para
/// que el mapa no crezca sin fin.
const CLEANUP_AT: usize = 10_000;

impl RateLimiter {
    pub fn new(max: u32, window: Duration) -> Self {
        RateLimiter { max, window, hits: Mutex::new(HashMap::new()) }
    }

    /// Cuenta la petición. `Err(segundos)` si se pasa: lo que falta para que
    /// se abra otra ventana.
    pub fn hit(&self, ip: IpAddr, now: Instant) -> Result<(), u64> {
        let mut hits = self.hits.lock().unwrap_or_else(|e| e.into_inner());
        if hits.len() >= CLEANUP_AT {
            let window = self.window;
            hits.retain(|_, (start, _)| now.duration_since(*start) < window);
        }
        let entry = hits.entry(ip).or_insert((now, 0));
        if now.duration_since(entry.0) >= self.window {
            *entry = (now, 0);
        }
        if entry.1 >= self.max {
            let wait = self.window.saturating_sub(now.duration_since(entry.0));
            return Err(wait.as_secs().max(1));
        }
        entry.1 += 1;
        Ok(())
    }
}

lazy_static! {
    /// Login de la tienda y del panel (es el mismo endpoint).
    static ref LOGIN: RateLimiter = RateLimiter::new(10, Duration::from_secs(5 * 60));
    /// Crear cuenta en la tienda y alta de vendedor.
    static ref REGISTER: RateLimiter = RateLimiter::new(5, Duration::from_secs(60 * 60));
    /// Pedir el enlace de "olvidé mi contraseña" (cada uno manda un email).
    static ref PASSWORD_FORGOT: RateLimiter = RateLimiter::new(5, Duration::from_secs(15 * 60));
    /// Enviar la nueva contraseña con el enlace, y cambiarla desde "Mi cuenta"
    /// (ahí se prueba la contraseña actual).
    static ref PASSWORD_CHANGE: RateLimiter = RateLimiter::new(10, Duration::from_secs(15 * 60));
}

fn minutes_label(secs: u64) -> String {
    let minutes = secs.div_ceil(60);
    if minutes <= 1 { "1 minuto".to_string() } else { format!("{} minutos", minutes) }
}

/// 429. Con htmx (login de la tienda) va el modal, que es lo que esa página
/// pinta; si no, una página mínima. El login del panel mira solo el código.
fn too_many(req: &ServiceRequest, wait_secs: u64) -> HttpResponse {
    let message = format!(
        "Demasiados intentos seguidos. Espera {} y vuelve a intentarlo.",
        minutes_label(wait_secs)
    );
    let body = if req.headers().contains_key("HX-Request") {
        MessageModal::new(&message).render().unwrap_or(message)
    } else {
        format!(
            "<!DOCTYPE html><html lang=\"es\"><head><meta charset=\"UTF-8\"><title>Demasiados intentos</title></head>\
             <body style=\"font-family:sans-serif;max-width:560px;margin:3rem auto;padding:0 1rem\">\
             <h1>Demasiados intentos</h1><p>{}</p><p><a href=\"javascript:history.back()\">Volver</a></p></body></html>",
            message
        )
    };
    HttpResponse::TooManyRequests()
        .insert_header((header::RETRY_AFTER, wait_secs.to_string()))
        .content_type("text/html; charset=utf-8")
        .body(body)
}

async fn limit(
    limiter: &RateLimiter,
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    // Sin IP (no pasa con TCP) no se limita: mejor eso que bloquear a todos juntos.
    if let Some(ip) = req.peer_addr().map(|a| a.ip()) {
        if let Err(wait) = limiter.hit(ip, Instant::now()) {
            let res = too_many(&req, wait);
            return Ok(req.into_response(res));
        }
    }
    Ok(next.call(req).await?.map_into_boxed_body())
}

pub async fn limit_login(req: ServiceRequest, next: Next<impl MessageBody + 'static>) -> Result<ServiceResponse<BoxBody>, Error> {
    limit(&LOGIN, req, next).await
}

pub async fn limit_register(req: ServiceRequest, next: Next<impl MessageBody + 'static>) -> Result<ServiceResponse<BoxBody>, Error> {
    limit(&REGISTER, req, next).await
}

pub async fn limit_password_forgot(req: ServiceRequest, next: Next<impl MessageBody + 'static>) -> Result<ServiceResponse<BoxBody>, Error> {
    limit(&PASSWORD_FORGOT, req, next).await
}

pub async fn limit_password_change(req: ServiceRequest, next: Next<impl MessageBody + 'static>) -> Result<ServiceResponse<BoxBody>, Error> {
    limit(&PASSWORD_CHANGE, req, next).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn blocks_after_max_and_reopens_after_window() {
        let limiter = RateLimiter::new(3, Duration::from_secs(60));
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let other = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
        let t0 = Instant::now();

        for _ in 0..3 {
            assert!(limiter.hit(ip, t0).is_ok());
        }
        assert_eq!(limiter.hit(ip, t0 + Duration::from_secs(20)), Err(40));
        // Otra IP lleva su propia cuenta.
        assert!(limiter.hit(other, t0).is_ok());
        // Pasada la ventana, vuelve a dejar.
        assert!(limiter.hit(ip, t0 + Duration::from_secs(60)).is_ok());
    }

    #[test]
    fn minutes_label_rounds_up() {
        assert_eq!(minutes_label(30), "1 minuto");
        assert_eq!(minutes_label(61), "2 minutos");
    }
}
