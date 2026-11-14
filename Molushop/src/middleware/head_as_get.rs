//! `HEAD` responde como `GET`, pero sin cuerpo (lo pide HTTP: RFC 9110, 9.3.2).
//!
//! Las rutas se registran con `web::get()`, que solo acepta `GET`, así que un
//! `HEAD` daba 404. Render hace un `HEAD /` en cada arranque para ver si el
//! servicio responde, y lo usan también monitores y algunos navegadores.
//!
//! Aquí solo se cambia el método antes de elegir la ruta: el cuerpo lo quita
//! actix-http al enviar, porque mira el método con el que llegó la petición
//! (HTTP/1 y HTTP/2). Así se conservan el estado, las cabeceras y el
//! `Content-Length` de verdad.
//!
//! Va lo más dentro posible en `main.rs` (el primer `.wrap`): los middlewares de
//! fuera (el log de peticiones) siguen viendo `HEAD`.

use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::Method;
use actix_web::middleware::Next;
use actix_web::Error;

pub async fn head_as_get(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    if req.method() == Method::HEAD {
        req.head_mut().method = Method::GET;
    }
    next.call(req).await
}
