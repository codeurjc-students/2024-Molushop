//! Conexiones del pool de la base de datos.

use diesel::result::{DatabaseErrorKind, Error};
use diesel_async::pooled_connection::deadpool::{Object, Pool};
use diesel_async::AsyncPgConnection;

type DbPool = Pool<AsyncPgConnection>;

/// Saca una conexión del pool. Si no se puede (pool agotado, base de datos
/// caída) devuelve un error de diesel en vez de hacer panic, para que el
/// servicio lo propague con `?` como cualquier otro fallo de la consulta.
pub async fn get_connection(pool: &DbPool) -> Result<Object<AsyncPgConnection>, Error> {
    pool.get().await.map_err(|e| {
        Error::DatabaseError(
            DatabaseErrorKind::ClosedConnection,
            Box::new(format!("Error obteniendo conexión: {:?}", e)),
        )
    })
}
