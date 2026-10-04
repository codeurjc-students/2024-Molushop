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
use crate::models::pages_models::panel_home_page_model::PanelHomePage;
use crate::models::components::panel_nav::PanelNav;
use crate::services::panel_role_service::{get_panel_user, PanelUser};
use crate::routes::panel_products;
use crate::routes::panel_register;
use crate::routes::admin_stores::{self, ADMIN_STORES};
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
        .configure(panel_register::config)
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
                .configure(admin_stores::config)
        );
}

#[derive(Deserialize)]
pub struct PanelLoginQuery {
    next: Option<String>,
    /// Viene de `/panel/register` tras crear la cuenta: se enseña el aviso.
    registered: Option<String>,
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
    let query = query.into_inner();
    let registered = query.registered.is_some();
    let next = safe_next(query.next);
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
            registered,
        },
    }.render().unwrap();

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(page)
}

/// Las tiendas del usuario. Quien solo gestiona una (y no es admin) va directo
/// a sus productos: la lista de una sola tienda no le aporta nada.
async fn panel_home(panel_user: web::ReqData<PanelUser>) -> HttpResponse {
    if !panel_user.is_admin && panel_user.stores.len() == 1 && panel_user.stores[0].is_manageable() {
        return HttpResponse::Found()
            .insert_header((header::LOCATION, format!("/panel/stores/{}/products", panel_user.stores[0].id)))
            .finish();
    }

    let page = PanelHomePage {
        user_logged: true,
        page_name: "Panel".to_string(),
        panel_nav: PanelNav::new(&panel_user, "home"),
        stores: panel_user.stores.clone(),
    }.render().unwrap();

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(page)
}

/// Lo único exclusivo del admin hoy es la gestión de tiendas.
async fn admin_home() -> HttpResponse {
    HttpResponse::Found()
        .insert_header((header::LOCATION, ADMIN_STORES))
        .finish()
}