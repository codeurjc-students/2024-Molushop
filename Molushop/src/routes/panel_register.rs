//! `/panel/register`: solicitar una tienda. Va FUERA del scope `/panel` (como
//! el login) porque quien llega aquí todavía no tiene acceso al panel.
//!
//! - Sin sesión: crea la cuenta y la tienda a la vez, y manda a iniciar sesión.
//! - Con sesión: solo pide el nombre de la tienda ("abrir mi tienda").
//! En los dos casos la tienda nace pendiente de aprobación.

use actix_web::{http::header, web, HttpResponse};
use askama::Template;
use serde::Deserialize;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::middleware::auth::{Auth, SessionData};
use crate::middleware::panel_guard::PANEL_LOGIN;
use crate::models::components::login_base_model::FormRegister;
use crate::models::pages_models::panel_register_page_model::PanelRegisterPage;
use crate::services::store_request_service::{
    get_username, register_seller, request_store_for_user, StoreRequestError,
};

pub static PANEL_REGISTER: &str = "/panel/register";

/// Hay que registrarlo ANTES que el scope `/panel`, igual que el login.
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource(PANEL_REGISTER)
            .wrap(Auth::new())
            .route(web::get().to(register_page))
            .route(web::post().to(register_submit)),
    );
}

#[derive(Deserialize, Default)]
pub struct RegisterForm {
    store_name: String,
    // Solo llegan cuando no hay sesión.
    username: Option<String>,
    email: Option<String>,
    pass: Option<String>,
    pass2: Option<String>,
}

fn html(status: actix_web::http::StatusCode, body: String) -> HttpResponse {
    HttpResponse::build(status)
        .content_type("text/html; charset=utf-8")
        .body(body)
}

fn redirect(location: String) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, location))
        .finish()
}

/// El formulario, vacío o con lo que se había escrito y el error. Las
/// contraseñas nunca se devuelven a la página.
async fn render(
    session: &Option<web::ReqData<SessionData>>,
    form: &RegisterForm,
    error: Option<&StoreRequestError>,
    pool: &DbPool,
) -> HttpResponse {
    let logged_username = match session {
        Some(session) => match get_username(&session.id, pool).await {
            Ok(username) => Some(username),
            Err(e) => {
                println!("Registro de tienda: {:?}", e);
                return html(
                    actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Error cargando la página".to_string(),
                );
            }
        },
        None => None,
    };

    let page = PanelRegisterPage {
        user_logged: logged_username.is_some(),
        page_name: "Panel - Solicitar tienda".to_string(),
        logged_username,
        error: error.map(|e| e.message()),
        store_name: form.store_name.clone(),
        username: form.username.clone().unwrap_or_default(),
        email: form.email.clone().unwrap_or_default(),
    }.render().unwrap();

    let status = match error {
        None => actix_web::http::StatusCode::OK,
        Some(StoreRequestError::Db(_)) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        Some(_) => actix_web::http::StatusCode::BAD_REQUEST,
    };
    html(status, page)
}

async fn register_page(
    pool_data: web::Data<DbPool>,
    opt_session_data: Option<web::ReqData<SessionData>>,
) -> HttpResponse {
    render(&opt_session_data, &RegisterForm::default(), None, pool_data.get_ref()).await
}

async fn register_submit(
    form: web::Form<RegisterForm>,
    pool_data: web::Data<DbPool>,
    opt_session_data: Option<web::ReqData<SessionData>>,
) -> HttpResponse {
    let form = form.into_inner();
    let pool = pool_data.get_ref();

    let result = match &opt_session_data {
        // Con sesión: la tienda es para quien la pide; el resto del formulario se ignora.
        Some(session) => request_store_for_user(&session.id, &form.store_name, pool)
            .await
            .map(|_| "/panel".to_string()),
        None => {
            let account = FormRegister {
                username: form.username.clone().unwrap_or_default().trim().to_string(),
                email: form.email.clone().unwrap_or_default().trim().to_string(),
                pass: form.pass.clone().unwrap_or_default(),
                pass2: form.pass2.clone().unwrap_or_default(),
            };
            register_seller(&account, &form.store_name, pool)
                .await
                .map(|_| format!("{}?registered=1", PANEL_LOGIN))
        }
    };

    match result {
        Ok(location) => redirect(location),
        Err(e) => {
            if let StoreRequestError::Db(detail) = &e {
                println!("Registro de tienda: {}", detail);
            }
            render(&opt_session_data, &form, Some(&e), pool).await
        }
    }
}
