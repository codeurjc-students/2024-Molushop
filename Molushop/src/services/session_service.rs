//! Abrir una sesión: el token PASETO, su fila en `user_sessions` y la cookie
//! `sesion_token`. Lo usan el login y el alta de vendedor, que entra solo tras
//! registrarse. Cerrarla es `revoke_session` (logout).

use std::collections::HashMap;
use std::time::Duration as DurationTime;

use actix_web::cookie::time::Duration as CookieDuration;
use actix_web::cookie::Cookie;
use actix_web::HttpRequest;
use chrono::{Duration, Utc};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use ipnet::IpNet;
use uuid::Uuid;

use crate::models::models_x::NewUserSession;
use crate::services::paseto_token_session_service::generate_local_token_duration;
use crate::services::servicesX::insert_session_login;

type DbPool = Pool<AsyncPgConnection>;

/// Lo que dura la sesión: el token, la fila y la cookie.
const SESSION_SECONDS: i64 = 3600;

/// Crea la sesión de `user_id` y devuelve la cookie que hay que poner en la
/// respuesta. Quien llama ya ha comprobado quién es (contraseña o alta recién
/// hecha). `Err` con el detalle para el log; al usuario no se le enseña.
pub async fn start_session(user_id: &Uuid, req: &HttpRequest, pool: &DbPool) -> Result<Cookie<'static>, String> {
    let jti = Uuid::new_v4().to_string();
    let mut claims = HashMap::new();
    claims.insert("iss".to_string(), "Molushop.com".to_string());
    claims.insert("aud".to_string(), "Molushop/user".to_string());
    claims.insert("sub".to_string(), user_id.to_string());
    claims.insert("jti".to_string(), jti.clone());

    let token = generate_local_token_duration(&DurationTime::from_secs(SESSION_SECONDS as u64), &claims)
        .map_err(|e| format!("Error generando el token: {}", e))?;

    let user_agent = req.headers().get("User-Agent").and_then(|v| v.to_str().ok());
    let ip: Option<IpNet> = req.peer_addr().map(|addr| addr.ip().into());
    let now = Utc::now().naive_utc();
    let expires_at = now
        .checked_add_signed(Duration::seconds(SESSION_SECONDS))
        .unwrap_or(now);

    let session = NewUserSession {
        id: &Uuid::new_v4(),
        user_id,
        jti: &jti,
        // Pendiente antes de publicar: guarda el token tal cual (la columna no se lee).
        refresh_token_hash: &token,
        issued_at: &now,
        expires_at: &expires_at,
        last_used_at: &now,
        is_revoked: &false,
        ip_address: ip.as_ref(),
        user_agent,
        device_info: Some("por ahora xd"),
    };
    insert_session_login(session, pool)
        .await
        .map_err(|e| format!("Error guardando la sesión: {}", e))?;

    let mut cookie = Cookie::new("sesion_token", token);
    cookie.set_max_age(CookieDuration::seconds(SESSION_SECONDS));
    cookie.set_http_only(true);
    cookie.set_secure(true);
    cookie.set_path("/");
    Ok(cookie)
}
