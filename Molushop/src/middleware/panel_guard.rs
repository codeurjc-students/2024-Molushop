use std::future::{ready, Ready};
use std::rc::Rc;
use actix_web::{
    body::EitherBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    web, Error, HttpMessage, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::middleware::auth::SessionData;
use crate::services::panel_role_service::get_panel_user;

/// A donde se manda a quien entra al panel sin sesión.
pub static PANEL_LOGIN: &str = "/panel/login";

/// Guardia de `/panel` y `/admin`. Va DETRÁS de `Auth`, que es quien mete
/// `SessionData`: en actix el último `.wrap()` es el que se ejecuta primero,
/// así que el scope tiene que quedar `.wrap(PanelGuard::...).wrap(Auth::new())`.
///
/// - Sin sesión: a `/panel/login` (302, o `HX-Redirect` si es petición htmx,
///   porque htmx metería la página de login dentro del fragmento).
/// - Con sesión y sin el rol pedido (o si falla la BD): 403.
/// - Si pasa: mete `PanelUser` en la petición. Los controladores lo reciben con
///   `web::ReqData<PanelUser>` y de ahí sacan el `seller_id`, en vez de fijarlo.
#[derive(Clone)]
pub struct PanelGuard {
    require_admin: bool,
}

impl PanelGuard {
    /// Admin o vendedor.
    pub fn panel() -> Self {
        Self { require_admin: false }
    }

    /// Solo admin.
    pub fn admin() -> Self {
        Self { require_admin: true }
    }
}

impl<S: 'static, B> Transform<S, ServiceRequest> for PanelGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = PanelGuardMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(PanelGuardMiddleware {
            service: Rc::new(service),
            require_admin: self.require_admin,
        }))
    }
}

pub struct PanelGuardMiddleware<S> {
    service: Rc<S>,
    require_admin: bool,
}

impl<S, B> Service<ServiceRequest> for PanelGuardMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let require_admin = self.require_admin;

        Box::pin(async move {
            let user_id = req.extensions().get::<SessionData>().map(|s| s.id);

            let user_id = match user_id {
                Some(id) => id,
                None => {
                    let res = redirect_to_login(&req);
                    return Ok(req.into_response(res).map_into_right_body());
                }
            };

            // Sin pool no se puede comprobar el rol: se deniega, no se deja pasar
            let panel_user = match req.app_data::<web::Data<DbPool>>() {
                Some(pool) => get_panel_user(&user_id, pool.get_ref()).await,
                None => Err("Sin pool en app_data".to_string()),
            };

            let allowed = match &panel_user {
                Ok(pu) if require_admin => pu.is_admin,
                Ok(pu) => pu.has_panel_access(),
                Err(e) => {
                    println!("PanelGuard: {}", e);
                    false
                }
            };

            if !allowed {
                let res = HttpResponse::Forbidden()
                    .content_type("text/html; charset=utf-8")
                    .body("<h1>403</h1><p>No tienes acceso a esta sección.</p>");
                return Ok(req.into_response(res).map_into_right_body());
            }

            // `allowed` solo es true con Ok
            req.extensions_mut().insert(panel_user.unwrap());

            let res = service.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}

/// Lleva al login recordando a dónde se quería ir (`?next=`), salvo en htmx:
/// ahí la ruta pedida es la de un fragmento, y volver a ella tras el login
/// enseñaría el fragmento suelto.
fn redirect_to_login(req: &ServiceRequest) -> HttpResponse {
    if req.headers().contains_key("HX-Request") {
        return HttpResponse::Unauthorized()
            .insert_header(("HX-Redirect", PANEL_LOGIN))
            .finish();
    }

    let next = req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or("/panel");
    let location = format!("{}?next={}", PANEL_LOGIN, urlencoding_path(next));

    HttpResponse::Found()
        .insert_header((header::LOCATION, location))
        .finish()
}

/// Codifica lo justo para meter una ruta local en un parámetro de query.
fn urlencoding_path(path: &str) -> String {
    path.replace('%', "%25")
        .replace('&', "%26")
        .replace('?', "%3F")
        .replace('=', "%3D")
        .replace('#', "%23")
        .replace(' ', "%20")
}
