//! "Mi cuenta" (`/account`): cambiar la contraseña y cerrar las demás sesiones.
//! Los dos POST son formularios clásicos que vuelven a `/account` con `?ok=` o
//! `?error=` (como en el panel). La lógica está en `account_service`.

use actix_web::{http::header, web, HttpResponse};
use askama::Template;
use serde::Deserialize;
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::constants::urls::ACCOUNT_URL;
use actix_web::middleware::from_fn;

use crate::middleware::auth::{Auth, SessionData};
use crate::middleware::rate_limit::limit_password_change;
use crate::models::pages_models::account_page_model::AccountPage;
use crate::services::account_service::{change_password, other_sessions_count, revoke_other_sessions};
use crate::services::components::login_base_service;
use crate::services::components::nav1_service::get_nav1_object;
use crate::services::servicesX::get_user_2;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(ACCOUNT_URL)
            .wrap(Auth::new())
            .route("", web::get().to(account_page))
            .route("/password", web::post().to(password_submit).wrap(from_fn(limit_password_change)))
            .route("/sessions", web::post().to(sessions_submit)),
    );
}

#[derive(Deserialize)]
pub struct AccountQuery {
    ok: Option<String>,
    /// Sesiones cerradas, para el aviso de `ok`.
    n: Option<usize>,
    error: Option<String>,
}

#[derive(Deserialize)]
pub struct PasswordForm {
    current: String,
    new: String,
    new2: String,
}

fn ok_message(code: &str, closed: usize) -> Option<String> {
    let sessions = match closed {
        0 => String::new(),
        1 => " Se ha cerrado 1 sesión en otro dispositivo.".to_string(),
        n => format!(" Se han cerrado {} sesiones en otros dispositivos.", n),
    };
    match code {
        "password" => Some(format!("Contraseña cambiada.{}", sessions)),
        "sessions" => Some(match closed {
            0 => "No había otras sesiones abiertas.".to_string(),
            _ => sessions.trim_start().to_string(),
        }),
        _ => None,
    }
}

fn error_message(code: &str) -> Option<&'static str> {
    match code {
        "current" => Some("La contraseña actual no es correcta."),
        "mismatch" => Some("Las dos contraseñas nuevas no coinciden."),
        "weak" => Some("La nueva contraseña no es segura: al menos 8 caracteres, con mayúscula, minúscula, número y un símbolo."),
        "same" => Some("La nueva contraseña es igual que la actual."),
        "db" => Some("Ha ocurrido un error. Inténtalo de nuevo."),
        _ => None,
    }
}

fn back_to_account(query: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((header::LOCATION, format!("{}?{}", ACCOUNT_URL, query)))
        .finish()
}

async fn account_page(
    query: web::Query<AccountQuery>,
    pool_data: web::Data<DbPool>,
    opt_session_data: Option<web::ReqData<SessionData>>,
) -> HttpResponse {
    let pool = pool_data.get_ref();
    let session = opt_session_data.map(|d| d.into_inner());
    let user_id: Option<Uuid> = session.as_ref().map(|s| s.id);

    let mut username = String::new();
    let mut user_logged = false;
    let mut other_sessions = 0;
    if let Some(session) = session.as_ref() {
        if let Ok(user) = get_user_2(&session.id, pool).await {
            username = user.username;
            user_logged = true;
        }
        other_sessions = other_sessions_count(&session.id, &session.jti, pool).await.unwrap_or_else(|e| {
            println!("Mi cuenta, sesiones: {:?}", e);
            0
        });
    }

    let query = query.into_inner();
    let page = AccountPage {
        user_logged,
        page_name: "Mi cuenta".to_string(),
        nav1: get_nav1_object(username.clone(), user_id.as_ref(), pool).await,
        login_base_data: login_base_service::get_login_base_model_data(),
        username,
        other_sessions,
        ok_message: query.ok.as_deref().and_then(|c| ok_message(c, query.n.unwrap_or(0))),
        error_message: query.error.as_deref().and_then(error_message),
    }.render().unwrap();

    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(page)
}

async fn password_submit(
    form: web::Form<PasswordForm>,
    pool_data: web::Data<DbPool>,
    opt_session_data: Option<web::ReqData<SessionData>>,
) -> HttpResponse {
    // Sin sesión, la página enseña el botón de identificarse.
    let Some(session) = opt_session_data.map(|d| d.into_inner()) else {
        return back_to_account("");
    };
    match change_password(&session.id, &session.jti, &form.current, &form.new, &form.new2, pool_data.get_ref()).await {
        Ok(closed) => back_to_account(&format!("ok=password&n={}", closed)),
        Err(e) => {
            println!("Mi cuenta, contraseña: {:?}", e);
            back_to_account(&format!("error={}", e.code()))
        }
    }
}

async fn sessions_submit(
    pool_data: web::Data<DbPool>,
    opt_session_data: Option<web::ReqData<SessionData>>,
) -> HttpResponse {
    let Some(session) = opt_session_data.map(|d| d.into_inner()) else {
        return back_to_account("");
    };
    match revoke_other_sessions(&session.id, &session.jti, pool_data.get_ref()).await {
        Ok(closed) => back_to_account(&format!("ok=sessions&n={}", closed)),
        Err(e) => {
            println!("Mi cuenta, sesiones: {:?}", e);
            back_to_account(&format!("error={}", e.code()))
        }
    }
}
