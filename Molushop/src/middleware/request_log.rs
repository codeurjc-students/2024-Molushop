//! Log de peticiones con un id por petición.
//!
//! - `TracingLogger` (de `tracing-actix-web`) da a cada petición un id (UUID)
//!   y abre un span con él: todo lo que se registre durante la petición sale
//!   con su `request_id`, su método y su ruta. El span lo monta `RequestSpan`.
//! - `request_log`, dentro de ese span, escribe una sola línea al acabar
//!   (código y tiempo) y devuelve el id en la cabecera `X-Request-Id`.
//! - La ruta va SIN la query (`/password/reset?token=` lleva el token) y no se
//!   registra ninguna cabecera (la cookie de sesión).
//! - Los estáticos (`/static/`, `/assets/`, `/css/`) llevan id, pero su span y
//!   su línea son de nivel `trace`: con el filtro de serie no salen.
//! - Los avisos de error (`MessageModal::with_error_ref`) enseñan el principio
//!   del id ("Error 500 · ref. 7f3a9c2e"), para buscar la petición en el log.
//!
//! En `main.rs`, `TracingLogger` tiene que ir por fuera de `request_log`
//! (envuelto después): `.wrap(from_fn(request_log)).wrap(TracingLogger::<RequestSpan>::new())`.

use std::time::Instant;

use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::middleware::Next;
use actix_web::{Error, HttpMessage};
use tracing::Span;
use tracing_actix_web::{RequestId, RootSpanBuilder};

tokio::task_local! {
    /// El id de la petición en curso, para los avisos de error, que se montan
    /// sin tener la petición a mano.
    static REQUEST_ID: RequestId;
}

const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// Cuántos caracteres del id se enseñan en los avisos: el primer bloque del UUID.
const REFERENCE_LEN: usize = 8;

/// ¿Es un fichero estático? (los sirve actix-files en `config::static_config`)
pub fn is_static(path: &str) -> bool {
    ["/static/", "/assets/", "/css/"].iter().any(|prefix| path.starts_with(prefix))
}

/// El principio del id de la petición en curso ("7f3a9c2e"). `None` fuera de
/// una petición (tests, tareas en segundo plano).
pub fn current_reference() -> Option<String> {
    REQUEST_ID
        .try_with(|id| {
            let mut reference = id.to_string();
            reference.truncate(REFERENCE_LEN);
            reference
        })
        .ok()
}

/// El span de cada petición: id, método y ruta sin query; nada de cabeceras.
pub struct RequestSpan;

impl RootSpanBuilder for RequestSpan {
    fn on_request_start(request: &ServiceRequest) -> Span {
        // TracingLogger mete el id en las extensiones antes de llamar aquí
        let request_id = request
            .extensions()
            .get::<RequestId>()
            .map(|id| id.to_string())
            .unwrap_or_default();
        let method = request.method();
        let path = request.path();
        if is_static(path) {
            tracing::trace_span!("request", request_id = %request_id, %method, path)
        } else {
            tracing::info_span!("request", request_id = %request_id, %method, path)
        }
    }

    // La línea final la escribe `request_log`, que sabe cuánto ha tardado
    fn on_request_end<B: MessageBody>(_span: Span, _outcome: &Result<ServiceResponse<B>, Error>) {}
}

/// Una línea por petición al acabar (código y milisegundos; el error, si lo
/// hubo) y la cabecera `X-Request-Id` en la respuesta.
pub async fn request_log(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let start = Instant::now();
    let static_file = is_static(req.path());
    let request_id = req.extensions().get::<RequestId>().copied();

    let mut outcome = match request_id {
        Some(id) => REQUEST_ID.scope(id, next.call(req)).await,
        None => next.call(req).await,
    };

    let duration_ms = start.elapsed().as_millis() as u64;
    let (status, error) = match &outcome {
        Ok(res) => (res.status(), res.response().error().map(|e| e.to_string())),
        Err(e) => (e.as_response_error().status_code(), Some(e.to_string())),
    };
    let status_code = status.as_u16();
    let error = error.as_deref();
    if static_file {
        tracing::trace!(status = status_code, duration_ms, error, "request finished");
    } else if status.is_server_error() {
        tracing::error!(status = status_code, duration_ms, error, "request finished");
    } else {
        tracing::info!(status = status_code, duration_ms, error, "request finished");
    }

    // Con Err no hay respuesta todavía (la monta actix): se queda sin cabecera
    if let (Ok(res), Some(id)) = (&mut outcome, request_id) {
        if let Ok(value) = HeaderValue::from_str(&id.to_string()) {
            res.headers_mut().insert(REQUEST_ID_HEADER, value);
        }
    }
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::test::{call_service, init_service, read_body, TestRequest};
    use actix_web::{web, App, HttpResponse};
    use tracing_actix_web::TracingLogger;

    use crate::utils::hx::error_modal;

    #[test]
    fn static_paths() {
        assert!(is_static("/assets/imgs/no-image.svg"));
        assert!(is_static("/static/base.js"));
        assert!(!is_static("/home"));
        assert!(!is_static("/assets"));
    }

    /// Fuera de una petición no hay referencia (y el aviso sale sin ella).
    #[test]
    fn no_reference_outside_a_request() {
        assert_eq!(current_reference(), None);
    }

    /// Cada respuesta lleva su X-Request-Id, y el aviso de error enseña su principio.
    #[actix_web::test]
    async fn response_has_request_id_and_modal_shows_its_reference() {
        let app = init_service(
            App::new()
                .wrap(actix_web::middleware::from_fn(request_log))
                .wrap(TracingLogger::<RequestSpan>::new())
                .route("/ok", web::get().to(|| async { HttpResponse::Ok().finish() }))
                .route("/fail", web::get().to(|| async { error_modal(StatusCode::INTERNAL_SERVER_ERROR, "Ha ocurrido un error") })),
        )
        .await;

        let ok = call_service(&app, TestRequest::get().uri("/ok").to_request()).await;
        let ok_id = ok.headers().get("X-Request-Id").expect("X-Request-Id").to_str().unwrap().to_string();
        assert_eq!(ok_id.len(), 36);

        let fail = call_service(&app, TestRequest::get().uri("/fail").to_request()).await;
        assert_eq!(fail.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let fail_id = fail.headers().get("X-Request-Id").unwrap().to_str().unwrap().to_string();
        assert_ne!(fail_id, ok_id);
        let body = String::from_utf8(read_body(fail).await.to_vec()).unwrap();
        assert!(body.contains(&format!("Error 500 · ref. {}", &fail_id[..REFERENCE_LEN])), "{body}");
    }
}
