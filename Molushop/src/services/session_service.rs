//! Abrir una sesión: el token PASETO, su fila en `user_sessions` y la cookie
//! `sesion_token`. Lo usan el login y el alta de vendedor, que entra solo tras
//! registrarse. Aquí también se comprueba (`check_session`, el middleware) y se
//! cierra (`revoke_session`, logout).

use std::collections::HashMap;
use std::time::Duration as DurationTime;

use actix_web::cookie::time::Duration as CookieDuration;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::HttpRequest;
use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel::result::Error;
use diesel::{insert_into, update};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use ipnet::IpNet;
use uuid::Uuid;

use crate::models::models_x::{NewUserSession, UserSession};
use crate::services::paseto_token_session_service::generate_local_token_duration;

use crate::utils::tokens::sha256_hex;

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
        // Nunca el token en claro: si se filtra la tabla, no sirve para entrar.
        // Hoy la columna no se lee; el hash permite buscar una sesión por su token.
        refresh_token_hash: &sha256_hex(&token),
        issued_at: &now,
        expires_at: &expires_at,
        last_used_at: &now,
        is_revoked: &false,
        ip_address: ip.as_ref(),
        user_agent,
        device_info: Some("por ahora xd"),
    };
    insert_session(session, pool)
        .await
        .map_err(|e| format!("Error guardando la sesión: {}", e))?;

    let mut cookie = Cookie::new("sesion_token", token);
    // Explícito y no el Lax por defecto del navegador: la cookie no viaja en
    // peticiones POST que vengan de otra web (primera barrera contra CSRF).
    cookie.set_same_site(SameSite::Lax);
    cookie.set_max_age(CookieDuration::seconds(SESSION_SECONDS));
    cookie.set_http_only(true);
    cookie.set_secure(true);
    cookie.set_path("/");
    Ok(cookie)
}

pub async fn insert_session<'a>(session_data: NewUserSession<'a>, pool:&DbPool) -> Result<usize, Error> {
    use crate::schema::user_sessions::dsl::*;
    let connection = &mut pool.get().await.unwrap();

    let result = insert_into(user_sessions)
        .values(session_data)
        .execute(connection).await;
    result
}

pub async fn check_session(jti_x:&String, pool:&DbPool)-> Result<UserSession,Error>{
    use crate::schema::user_sessions::dsl::*;
    let connection = &mut pool.get().await.unwrap();

    //let now = Utc::now().naive_utc();

    let result = user_sessions
        .filter(jti.eq(jti_x))
        // .filter(is_revoked.eq(false))
        // .filter(issued_at.lt(now))
        // .filter(expires_at.gt(now))
        .first::<UserSession>(connection).await;
    result
}

/// Revoca la sesión `jti_x`. Se filtra también por usuario para que nunca pueda revocarse
/// una sesión ajena, aunque el `jti` llegue de otro sitio que no sea el middleware.
pub async fn revoke_session(jti_x:&String, user_id_x:&Uuid, pool:&DbPool)-> Result<usize,Error>{
    use crate::schema::user_sessions::dsl::*;
    let connection = &mut pool.get().await.unwrap();

    update(user_sessions.filter(jti.eq(jti_x)).filter(user_id.eq(user_id_x)))
        .set(is_revoked.eq(true))
        .execute(connection).await
}
