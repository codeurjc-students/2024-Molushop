//! "Mi cuenta" (`/account`): cambiar la contraseña y cerrar las demás sesiones.
//!
//! Las sesiones viven en `user_sessions` (una fila por login, ver
//! `session_service`); "cerrar" una es marcarla `is_revoked`, que es lo que
//! comprueba el middleware en cada petición. La sesión desde la que se pide
//! (`current_jti`) se respeta siempre: quien cambia su contraseña sigue dentro.

use chrono::Utc;
use diesel::prelude::*;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::schema::{base_user, user_sessions};
use crate::services::auth::{hash_password, verify_password};
use crate::utils::validations::is_password_strong;

type DbPool = Pool<AsyncPgConnection>;

#[derive(Debug)]
pub enum AccountError {
    WrongCurrentPassword,
    PasswordMismatch,
    PasswordWeak,
    /// La nueva es igual que la actual.
    SamePassword,
    Db(String),
}

impl AccountError {
    /// Código corto para llevar el error en la URL tras el POST (`?error=...`).
    pub fn code(&self) -> &'static str {
        match self {
            AccountError::WrongCurrentPassword => "current",
            AccountError::PasswordMismatch => "mismatch",
            AccountError::PasswordWeak => "weak",
            AccountError::SamePassword => "same",
            AccountError::Db(_) => "db",
        }
    }
}

impl From<diesel::result::Error> for AccountError {
    fn from(e: diesel::result::Error) -> Self {
        AccountError::Db(format!("{:?}", e))
    }
}

async fn connection(
    pool: &DbPool,
) -> Result<diesel_async::pooled_connection::deadpool::Object<AsyncPgConnection>, AccountError> {
    pool.get()
        .await
        .map_err(|e| AccountError::Db(format!("Error obteniendo conexión: {:?}", e)))
}

/// Las reglas de la nueva contraseña, sin tocar la base de datos.
fn check_new_password(current: &str, new: &str, new2: &str) -> Result<(), AccountError> {
    if new != new2 {
        return Err(AccountError::PasswordMismatch);
    }
    if !is_password_strong(&new.to_string()) {
        return Err(AccountError::PasswordWeak);
    }
    if new == current {
        return Err(AccountError::SamePassword);
    }
    Ok(())
}

/// Cambia la contraseña si `current` es la de ahora y, en la misma
/// transacción, cierra las demás sesiones: si alguien la conocía y estaba
/// dentro, deja de estarlo. Devuelve cuántas se han cerrado.
pub async fn change_password(
    user_id: &Uuid,
    current_jti: &str,
    current: &str,
    new: &str,
    new2: &str,
    pool: &DbPool,
) -> Result<usize, AccountError> {
    check_new_password(current, new, new2)?;

    let conn = &mut connection(pool).await?;
    let stored_hash: String = base_user::table
        .find(user_id)
        .select(base_user::password)
        .first(conn)
        .await?;
    let ok = verify_password(current, &stored_hash)
        .map_err(|e| AccountError::Db(format!("Error comprobando la contraseña: {}", e)))?;
    if !ok {
        return Err(AccountError::WrongCurrentPassword);
    }

    let new_hash = hash_password(new)
        .map_err(|e| AccountError::Db(format!("Error hasheando la contraseña: {}", e)))?;

    conn.transaction::<usize, AccountError, _>(|conn| async move {
        diesel::update(base_user::table.find(user_id))
            .set(base_user::password.eq(&new_hash))
            .execute(conn)
            .await?;
        revoke_others(user_id, current_jti, conn).await
    }.scope_boxed())
    .await
}

/// "Cerrar sesión en todos los demás dispositivos". Devuelve cuántas se han cerrado.
pub async fn revoke_other_sessions(user_id: &Uuid, current_jti: &str, pool: &DbPool) -> Result<usize, AccountError> {
    let conn = &mut connection(pool).await?;
    revoke_others(user_id, current_jti, conn).await
}

async fn revoke_others(user_id: &Uuid, current_jti: &str, conn: &mut AsyncPgConnection) -> Result<usize, AccountError> {
    Ok(diesel::update(
        user_sessions::table
            .filter(user_sessions::user_id.eq(user_id))
            .filter(user_sessions::jti.ne(current_jti))
            .filter(user_sessions::is_revoked.eq(false)),
    )
    .set(user_sessions::is_revoked.eq(true))
    .execute(conn)
    .await?)
}

/// Sesiones abiertas aparte de la actual: no revocadas y sin caducar.
pub async fn other_sessions_count(user_id: &Uuid, current_jti: &str, pool: &DbPool) -> Result<i64, AccountError> {
    let conn = &mut connection(pool).await?;
    let now = Utc::now().naive_utc();
    Ok(user_sessions::table
        .filter(user_sessions::user_id.eq(user_id))
        .filter(user_sessions::jti.ne(current_jti))
        .filter(user_sessions::is_revoked.eq(false))
        .filter(user_sessions::expires_at.gt(now))
        .count()
        .get_result(conn)
        .await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_password_rules() {
        assert!(matches!(check_new_password("Vieja-123", "Nueva-123", "Nueva-124"), Err(AccountError::PasswordMismatch)));
        assert!(matches!(check_new_password("Vieja-123", "corta", "corta"), Err(AccountError::PasswordWeak)));
        assert!(matches!(check_new_password("Vieja-123", "Vieja-123", "Vieja-123"), Err(AccountError::SamePassword)));
        assert!(check_new_password("Vieja-123", "Nueva-123", "Nueva-123").is_ok());
    }
}
