use actix_web::{http::header, web, HttpResponse};
use askama::Template;
use serde::Deserialize;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::controllers::components::login_base_controller::ROUTES as LOGIN_ROUTES;
use crate::middleware::auth::{Auth, SessionData};
use crate::middleware::panel_guard::{PanelGuard, PANEL_LOGIN};
use crate::models::components::panel_login_v1::PanelLoginData;
use crate::models::pages_models::panel_login_page_model::PanelLoginPage;
use crate::services::panel_role_service::{get_panel_user, PanelUser};
use crate::routes::panel_products;
use crate::controllers::components::scope::scope_panel_components;
use crate::controllers::createProduct::create_product_controllers::scope_create_product;

/// `/panel`: vendedores y admin (el vendedor ve lo suyo, el admin todo).
/// `/admin`: solo lo exclusivo del admin.
///
/// Orden de los wraps: el último se ejecuta primero, así que `Auth` mete la
/// sesión y después `PanelGuard` comprueba el rol.
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        // ANTES que el scope `/panel` y fuera de él: si no, el scope lo captura
        // y el guardia no deja llegar al login a quien no tiene sesión.
        .route(PANEL_LOGIN, web::get().to(panel_login).wrap(Auth::new()))
        .service(
            web::scope("/panel")
                .wrap(PanelGuard::panel())
                .wrap(Auth::new())
                .route("", web::get().to(panel_home))
                .configure(panel_products::config)
                .service(scope_panel_components())
                .service(scope_create_product())
)
        .service(
            web::scope("/admin")
                .wrap(PanelGuard::admin())
                .wrap(Auth::new())
                .route("", web::get().to(admin_home))
        );
}

#[derive(Deserialize)]
pub struct PanelLoginQuery {
    next: Option<String>,
}

/// Solo se vuelve a rutas del panel. Sin esto, `?next=https://otra-web` (o
/// `//otra-web`) convertiría el login en un redirector abierto.
fn safe_next(next: Option<String>) -> String {
    match next {
        Some(n) if (n == "/panel" || n.starts_with("/panel/") || n.starts_with("/panel?")
                 || n == "/admin" || n.starts_with("/admin/") || n.starts_with("/admin?"))
                 && n != PANEL_LOGIN && !n.starts_with("/panel/login?") => n,
        _ => "/panel".to_string(),
    }
}

/// Con sesión y rol: directo a `next` (también es lo que pasa al recargar tras
/// un login correcto). Con sesión sin rol: el formulario con el aviso. Sin
/// sesión: el formulario.
async fn panel_login(
    query: web::Query<PanelLoginQuery>,
    pool_data: web::Data<DbPool>,
    opt_session_data: Option<web::ReqData<SessionData>>
) -> HttpResponse {
    let next = safe_next(query.into_inner().next);
    let mut logged_without_access = false;

    if let Some(session) = opt_session_data {
        match get_panel_user(&session.id, pool_data.get_ref()).await {
            Ok(pu) if pu.has_panel_access() => {
                return HttpResponse::Found()
                    .insert_header((header::LOCATION, next))
                    .finish();
            }
            Ok(_) => logged_without_access = true,
            Err(e) => println!("Login del panel: {}", e),
        }
    }

    let page = PanelLoginPage {
        user_logged: logged_without_access,
        page_name: "Panel - Entrar".to_string(),
        panel_login: PanelLoginData {
            login_url: LOGIN_ROUTES.login.to_string(),
            next,
            logged_without_access,
        },
    }.render().unwrap();

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(page)
}

// Provisionales: el paso 6 las sustituye por el layout del panel.

async fn panel_home(panel_user: web::ReqData<PanelUser>) -> HttpResponse {
    let pu = panel_user.into_inner();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!(
            "<h1>Panel (provisional)</h1><p>user_id: {}</p><p>admin: {}</p><p>seller_id: {:?}</p>",
            pu.user_id, pu.is_admin, pu.seller_id
        ))
}

async fn admin_home(panel_user: web::ReqData<PanelUser>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!("<h1>Admin (provisional)</h1><p>user_id: {}</p>", panel_user.user_id))
}
