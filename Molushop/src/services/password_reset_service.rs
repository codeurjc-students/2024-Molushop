//! "He olvidado mi contraseña".
//!
//! 1. `request_reset(email)`: si hay una cuenta con ese email, crea un token de
//!    un solo uso (caduca en `RESET_MINUTES`) y manda el enlace por email
//!    (`mail_service`). Si no la hay, no hace nada y responde igual: la página
//!    no revela qué emails tienen cuenta.
//! 2. `token_is_valid(token)`: para enseñar el formulario o "enlace caducado".
//! 3. `reset_password(token, nueva, nueva2)`: cambia la contraseña, gasta el
//!    token y cierra TODAS las sesiones de la cuenta.
//!
//! En la tabla `password_resets` solo está el SHA-256 del token.

use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::schema::{base_user, password_resets, user_sessions};
use crate::services::auth::hash_password;
use crate::services::mail_service::{public_base_url, send_password_reset};
use crate::utils::tokens::{random_token, sha256_hex};
use crate::utils::validations::is_password_strong;

type DbPool = Pool<AsyncPgConnection>;

diesel::define_sql_function!(fn lower(x: diesel::sql_types::Text) -> diesel::sql_types::Text);

/// Lo que dura el enlace.
pub const RESET_MINUTES: i64 = 30;
/// Ruta del formulario de la nueva contraseña (el enlace del email).
pub const RESET_URL: &str = "/password/reset";

#[derive(Debug)]
pub enum ResetError {
    /// No existe, ya se usó o ha caducado.
    InvalidToken,
    PasswordMismatch,
    PasswordWeak,
    Db(String),
}

impl ResetError {
    /// Código corto para llevar el error en la URL tras el POST (`?error=...`).
    pub fn code(&self) -> &'static str {
        match self {
            ResetError::InvalidToken => "token",
            ResetError::PasswordMismatch => "mismatch",
            ResetError::PasswordWeak => "weak",
            ResetError::Db(_) => "db",
        }
    }
}

impl From<diesel::result::Error> for ResetError {
    fn from(e: diesel::result::Error) -> Self {
        ResetError::Db(format!("{:?}", e))
    }
}

async fn connection(
    pool: &DbPool,
) -> Result<diesel_async::pooled_connection::deadpool::Object<AsyncPgConnection>, ResetError> {
    pool.get()
        .await
        .map_err(|e| ResetError::Db(format!("Error obteniendo conexión: {:?}", e)))
}

/// Paso 1. `Ok(())` tanto si la cuenta existe como si no. Un fallo al mandar el
/// email solo se apunta en el log: el usuario puede volver a pedirlo.
pub async fn request_reset(email: &str, pool: &DbPool) -> Result<(), ResetError> {
    let email = email.trim();
    if email.is_empty() {
        return Ok(());
    }
    let conn = &mut connection(pool).await?;

    let user: Option<(Uuid, String, String)> = base_user::table
        // Sin distinguir mayúsculas: "Ana@Mail.com" es la misma cuenta.
        .filter(lower(base_user::email).eq(lower(email)))
        .select((base_user::id, base_user::username, base_user::email))
        .first(conn)
        .await
        .optional()?;
    let Some((user_id, username, user_email)) = user else {
        return Ok(());
    };

    let token = random_token();
    let token_hash = sha256_hex(&token);
    let now = Utc::now().naive_utc();
    let expires_at = now + Duration::minutes(RESET_MINUTES);

    conn.transaction::<(), ResetError, _>(|conn| async move {
        // Solo vale el último enlace pedido.
        diesel::update(
            password_resets::table
                .filter(password_resets::user_id.eq(user_id))
                .filter(password_resets::used_at.is_null()),
        )
        .set(password_resets::used_at.eq(now))
        .execute(conn)
        .await?;

        diesel::insert_into(password_resets::table)
            .values((
                password_resets::user_id.eq(user_id),
                password_resets::token_hash.eq(&token_hash),
                password_resets::created_at.eq(now),
                password_resets::expires_at.eq(expires_at),
            ))
            .execute(conn)
            .await?;
        Ok(())
    }.scope_boxed())
    .await?;

    let link = format!("{}{}?token={}", public_base_url(), RESET_URL, token);
    if let Err(e) = send_password_reset(&user_email, &username, &link, RESET_MINUTES).await {
        println!("Recuperar contraseña, email: {:?}", e);
    }
    Ok(())
}

/// Paso 2. Sin usar y sin caducar.
pub async fn token_is_valid(token: &str, pool: &DbPool) -> Result<bool, ResetError> {
    let conn = &mut connection(pool).await?;
    Ok(diesel::select(diesel::dsl::exists(
        password_resets::table
            .filter(password_resets::token_hash.eq(sha256_hex(token)))
            .filter(password_resets::used_at.is_null())
            .filter(password_resets::expires_at.gt(Utc::now().naive_utc())),
    ))
    .get_result::<bool>(conn)
    .await?)
}

/// Las reglas de la nueva contraseña, sin tocar la base de datos.
fn check_new_password(new: &str, new2: &str) -> Result<(), ResetError> {
    if new != new2 {
        return Err(ResetError::PasswordMismatch);
    }
    if !is_password_strong(&new.to_string()) {
        return Err(ResetError::PasswordWeak);
    }
    Ok(())
}

/// Paso 3. Todo en una transacción y con la fila del token bloqueada (`FOR
/// UPDATE`): dos envíos a la vez del mismo enlace no pueden usarlo dos veces.
pub async fn reset_password(token: &str, new: &str, new2: &str, pool: &DbPool) -> Result<(), ResetError> {
    check_new_password(new, new2)?;
    let new_hash = hash_password(new)
        .map_err(|e| ResetError::Db(format!("Error hasheando la contraseña: {}", e)))?;
    let token_hash = sha256_hex(token);

    let conn = &mut connection(pool).await?;
    conn.transaction::<(), ResetError, _>(|conn| async move {
        let now = Utc::now().naive_utc();
        let (reset_id, user_id): (Uuid, Uuid) = password_resets::table
            .filter(password_resets::token_hash.eq(&token_hash))
            .filter(password_resets::used_at.is_null())
            .filter(password_resets::expires_at.gt(now))
            .select((password_resets::id, password_resets::user_id))
            .for_update()
            .first(conn)
            .await
            .optional()?
            .ok_or(ResetError::InvalidToken)?;

        diesel::update(base_user::table.find(user_id))
            .set(base_user::password.eq(&new_hash))
            .execute(conn)
            .await?;
        diesel::update(password_resets::table.find(reset_id))
            .set(password_resets::used_at.eq(now))
            .execute(conn)
            .await?;
        // Quien la ha olvidado no está dentro en ningún sitio que quiera
        // conservar, y si alguien más la sabía, deja de estar dentro.
        diesel::update(
            user_sessions::table
                .filter(user_sessions::user_id.eq(user_id))
                .filter(user_sessions::is_revoked.eq(false)),
        )
        .set(user_sessions::is_revoked.eq(true))
        .execute(conn)
        .await?;
        Ok(())
    }.scope_boxed())
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_password_rules() {
        assert!(matches!(check_new_password("Nueva-123", "Nueva-124"), Err(ResetError::PasswordMismatch)));
        assert!(matches!(check_new_password("corta", "corta"), Err(ResetError::PasswordWeak)));
        assert!(check_new_password("Nueva-123", "Nueva-123").is_ok());
    }
}
