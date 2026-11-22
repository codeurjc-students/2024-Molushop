//! Un trozo de la URL que no se puede leer (`/orders/abc` con `web::Path<Uuid>`)
//! responde como una URL que no existe, con la 404 de la tienda.
//!
//! Sin esto, actix respondía un 404 en texto plano con su error de parseo
//! ("UUID parsing failed: invalid character..."): feo y con detalles internos.
//! Son dos piezas:
//! - `path_config` (en `main.rs`, `app_data`): el error de `web::Path` pasa a ser
//!   un 404 sin cuerpo con la marca `InvalidPath` en la respuesta.
//! - `invalid_path_not_found` (middleware): si la respuesta trae la marca, la
//!   cambia por la de `routes::not_found::not_found_response`, la misma que la
//!   de la ruta por defecto (página solo para un GET de página; lo demás, 404
//!   sin cuerpo).
//!
//! Así vale para cualquier ruta con un id en la URL, también las que se añadan.
//! Las que tienen su propia 404 (la ficha de producto, una tienda) leen el id
//! como texto y no pasan por aquí.

use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::InternalError;
use actix_web::middleware::Next;
use actix_web::{web, Error, HttpRequest, HttpResponse};

use crate::routes::not_found::not_found_response;

/// Marca en la respuesta: la ha dado `path_config` por un trozo de URL inválido.
#[derive(Clone, Copy, Debug)]
struct InvalidPath;

/// Para `App::app_data`: vale para todos los `web::Path` de la aplicación.
pub fn path_config() -> web::PathConfig {
    web::PathConfig::default().error_handler(|err, req: &HttpRequest| {
        // debug: lo provoca quien escribe la URL a mano, no es un fallo nuestro
        tracing::debug!(path = %req.path(), error = %err, "invalid path segment");
        let mut response = HttpResponse::NotFound().finish();
        response.extensions_mut().insert(InvalidPath);
        InternalError::from_response(err, response).into()
    })
}

pub async fn invalid_path_not_found(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let res = next.call(req).await?;
    if res.response().extensions().get::<InvalidPath>().is_none() {
        return Ok(res.map_into_left_body());
    }
    let (req, _) = res.into_parts();
    let page = not_found_response(&req).await;
    Ok(ServiceResponse::new(req, page).map_into_right_body())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use uuid::Uuid;

    async fn echo(path: web::Path<Uuid>) -> HttpResponse {
        HttpResponse::Ok().body(path.into_inner().to_string())
    }

    /// Sin pool en la app (como aquí) not_found_response da el 404 sin cuerpo:
    /// basta para ver que ya no sale el texto de error de actix.
    #[actix_web::test]
    async fn invalid_uuid_gives_a_plain_not_found_without_the_parse_error() {
        let app = test::init_service(
            App::new()
                .wrap(actix_web::middleware::from_fn(invalid_path_not_found))
                .app_data(path_config())
                .route("/items/{id}", web::get().to(echo))
                .route("/items/{id}", web::post().to(echo)),
        ).await;

        for req in [
            test::TestRequest::get().uri("/items/not-a-uuid").to_request(),
            test::TestRequest::post().uri("/items/not-a-uuid").to_request(),
        ] {
            let res = test::call_service(&app, req).await;
            assert_eq!(res.status(), 404);
            let body = test::read_body(res).await;
            assert!(body.is_empty(), "cuerpo inesperado: {:?}", body);
        }
    }

    /// Un id válido llega a la ruta sin cambios.
    #[actix_web::test]
    async fn valid_uuid_reaches_the_handler() {
        let app = test::init_service(
            App::new()
                .wrap(actix_web::middleware::from_fn(invalid_path_not_found))
                .app_data(path_config())
                .route("/items/{id}", web::get().to(echo)),
        ).await;

        let id = Uuid::nil();
        let res = test::call_service(&app, test::TestRequest::get().uri(&format!("/items/{id}")).to_request()).await;
        assert_eq!(res.status(), 200);
        assert_eq!(test::read_body(res).await, id.to_string());
    }
}
