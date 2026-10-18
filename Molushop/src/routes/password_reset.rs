//! "He olvidado mi contraseña": `/password/forgot` (pedir el enlace) y
//! `/password/reset?token=...` (el enlace del email: elegir la nueva). Los POST
//! son formularios clásicos que vuelven a su página con `?...`. La lógica está
//! en `password_reset_service`; el envío del email, en `mail_service`.

use actix_web::{http::header, web, HttpResponse};
use askama::Template;
use serde::Deserialize;
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::middleware::auth::{Auth, SessionData};
use crate::models::pages_models::password_forgot_page_model::PasswordForgotPage;
use crate::models::pages_models::password_reset_page_model::PasswordResetPage;
use crate::services::components::login_base_service;
use crate::services::components::nav1_service::get_nav1_object;
use crate::services::password_reset_service::{
    request_reset, reset_password, token_is_valid, ResetError, RESET_MINUTES, RESET_URL,
};
use crate::services::servicesX::get_user_2;

pub static FORGOT_URL: &str = "/password/forgot";

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource(FORGOT_URL)
            .wrap(Auth::new())
            .route(web::get().to(forgot_page))
            .route(web::post().to(forgot_submit)),
    )
    .service(
        web::resource(RESET_URL)
            .wrap(Auth::new())
            .route(web::get().to(reset_page))
            .route(web::post().to(reset_submit)),
    );
}

#[derive(Deserialize)]
pub struct ForgotQuery {
    sent: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
pub struct ForgotForm {
    email: String,
}

#[derive(Deserialize)]
pub struct ResetQuery {
    token: Option<String>,
    done: Option<String>,
    error: Option<String>,
}

#[derive(Deserialize)]
pub struct ResetForm {
    token: String,
    new: String,
    new2: String,
}

fn error_message(code: &str) -> Option<&'static str> {
    match code {
        "mismatch" => Some("Las dos contraseñas no coinciden."),
        "weak" => Some("La contraseña no es segura: al menos 8 caracteres, con mayúscula, minúscula, número y un símbolo."),
        "db" => Some("Ha ocurrido un error. Inténtalo de nuevo."),
        _ => None,
    }
}

fn redirect(location: String) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, location))
        .finish()
}

fn html(page: String) -> HttpResponse {
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(page)
}

/// Lo que piden la barra y el login de todas las páginas de la tienda.
async fn nav_parts(
    opt_session_data: Option<web::ReqData<SessionData>>,
    pool: &DbPool,
) -> (bool, crate::models::components::nav1_model::Nav1Data) {
    let user_id: Option<Uuid> = opt_session_data.map(|d| d.into_inner().id);
    let mut username = String::new();
    let mut logged = false;
    if let Some(id) = user_id.as_ref() {
        if let Ok(user) = get_user_2(id, pool).await {
            username = user.username;
            logged = true;
        }
    }
    (logged, get_nav1_object(username, user_id.as_ref(), pool).await)
}

async fn forgot_page(
    query: web::Query<ForgotQuery>,
    pool_data: web::Data<DbPool>,
    opt_session_data: Option<web::ReqData<SessionData>>,
) -> HttpResponse {
    let pool = pool_data.get_ref();
    let (user_logged, nav1) = nav_parts(opt_session_data, pool).await;
    html(PasswordForgotPage {
        user_logged,
        page_name: "He olvidado mi contraseña".to_string(),
        nav1,
        login_base_data: login_base_service::get_login_base_model_data(),
        sent: query.sent.is_some(),
        reset_minutes: RESET_MINUTES,
        error_message: query.error.as_deref().and_then(error_message),
    }.render().unwrap())
}

async fn forgot_submit(form: web::Form<ForgotForm>, pool_data: web::Data<DbPool>) -> HttpResponse {
    match request_reset(&form.email, pool_data.get_ref()).await {
        Ok(()) => redirect(format!("{}?sent=1", FORGOT_URL)),
        Err(e) => {
            println!("Recuperar contraseña: {:?}", e);
            redirect(format!("{}?error={}", FORGOT_URL, e.code()))
        }
    }
}

async fn reset_page(
    query: web::Query<ResetQuery>,
    pool_data: web::Data<DbPool>,
    opt_session_data: Option<web::ReqData<SessionData>>,
) -> HttpResponse {
    let pool = pool_data.get_ref();
    let query = query.into_inner();
    let token = query.token.unwrap_or_default();
    let token_valid = !token.is_empty()
        && token_is_valid(&token, pool).await.unwrap_or_else(|e| {
            println!("Recuperar contraseña, token: {:?}", e);
            false
        });
    let (user_logged, nav1) = nav_parts(opt_session_data, pool).await;
    html(PasswordResetPage {
        user_logged,
        page_name: "Nueva contraseña".to_string(),
        nav1,
        login_base_data: login_base_service::get_login_base_model_data(),
        token,
        token_valid,
        done: query.done.is_some(),
        error_message: query.error.as_deref().and_then(error_message),
    }.render().unwrap())
}

async fn reset_submit(form: web::Form<ResetForm>, pool_data: web::Data<DbPool>) -> HttpResponse {
    match reset_password(&form.token, &form.new, &form.new2, pool_data.get_ref()).await {
        Ok(()) => redirect(format!("{}?done=1", RESET_URL)),
        // Sin token en la URL: la página dice que el enlace no vale.
        Err(ResetError::InvalidToken) => redirect(RESET_URL.to_string()),
        Err(e) => {
            println!("Recuperar contraseña: {:?}", e);
            // El token ya viajaba en la URL del email: volver con él no expone nada nuevo.
            redirect(format!("{}?token={}&error={}", RESET_URL, form.token, e.code()))
        }
    }
}
