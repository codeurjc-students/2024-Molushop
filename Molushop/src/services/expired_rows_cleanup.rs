//! Borrado periódico de filas caducadas: `user_sessions` (cada login deja una) y
//! `password_resets`. Sin esto las dos tablas crecen sin límite.
//!
//! Se guardan `RETENTION_DAYS` después de caducar para poder revisar un acceso raro
//! (IP, user agent). Las sesiones revocadas (logout) también caducan en una hora,
//! así que caen con la misma regla.
//!
//! La primera pasada es al arrancar: Render gratis duerme el servidor y puede que
//! nunca llegue a cumplirse el intervalo.

use std::time::Duration as StdDuration;

use chrono::{Duration, NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::schema::{password_resets, user_sessions};
use crate::utils::db::get_connection;

type DbPool = Pool<AsyncPgConnection>;

/// Días que se guarda una fila después de caducar.
const RETENTION_DAYS: i64 = 7;

/// Cada cuánto se repite el borrado.
const CLEANUP_EVERY: StdDuration = StdDuration::from_secs(60 * 60);

/// Lanza la tarea en segundo plano: una pasada ahora y luego cada `CLEANUP_EVERY`.
pub fn spawn(pool: DbPool) {
    actix_web::rt::spawn(async move {
        let mut ticker = actix_web::rt::time::interval(CLEANUP_EVERY);
        // Si el servidor se duerme, al despertar basta con una pasada, no con todas las perdidas.
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            // El primer tick es inmediato: la pasada del arranque.
            ticker.tick().await;
            run_once(&pool).await;
        }
    });
}

/// Una pasada: borra y lo escribe en el log (nada si no había qué borrar).
async fn run_once(pool: &DbPool) {
    let cutoff = cutoff(Utc::now().naive_utc());
    match delete_expired(cutoff, pool).await {
        Ok((0, 0)) => {}
        Ok((sessions, resets)) => tracing::info!(
            sessions,
            password_resets = resets,
            retention_days = RETENTION_DAYS,
            "expired rows deleted"
        ),
        Err(error) => tracing::error!(%error, "could not delete expired rows, retrying on the next run"),
    }
}

/// Lo que caducó antes de esta fecha se borra. Las fechas se guardan en UTC.
fn cutoff(now_utc: NaiveDateTime) -> NaiveDateTime {
    now_utc - Duration::days(RETENTION_DAYS)
}

/// Borra las sesiones y los enlaces de cambio de contraseña caducados antes de
/// `cutoff`. Devuelve cuántas filas de cada tabla.
async fn delete_expired(cutoff: NaiveDateTime, pool: &DbPool) -> Result<(usize, usize), Error> {
    let connection = &mut get_connection(pool).await?;
    let sessions = diesel::delete(user_sessions::table.filter(user_sessions::expires_at.lt(cutoff)))
        .execute(connection)
        .await?;
    let resets = diesel::delete(password_resets::table.filter(password_resets::expires_at.lt(cutoff)))
        .execute(connection)
        .await?;
    Ok((sessions, resets))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn cutoff_is_retention_days_before_now() {
        let now = NaiveDate::from_ymd_opt(2026, 11, 14).unwrap().and_hms_opt(12, 0, 0).unwrap();
        let expected = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap().and_hms_opt(12, 0, 0).unwrap();
        assert_eq!(cutoff(now), expected);
    }
}
