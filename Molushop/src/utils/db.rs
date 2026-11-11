//! Conexiones del pool de la base de datos.
//!
//! TLS: las conexiones se abren con `tokio-postgres` + rustls, y el `sslmode`
//! de la URL decide:
//! - sin `sslmode` (= `prefer`, el `.env` local): intenta TLS y, si el servidor
//!   no lo tiene (el Postgres local), sigue sin cifrar;
//! - `sslmode=require` (Neon): TLS obligatorio; sin él, no conecta.
//!   Neon añade `channel_binding=require`, que también se respeta.
//!
//! Los certificados raíz van dentro del binario (`webpki-roots`): la imagen
//! Docker no necesita el paquete `ca-certificates` para esto.

use std::env;
use std::sync::{Arc, OnceLock};

use diesel::result::{DatabaseErrorKind, Error};
use diesel::{ConnectionError, ConnectionResult};
use diesel_async::pooled_connection::deadpool::{Object, Pool};
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use diesel_async::AsyncPgConnection;
use futures_util::future::{BoxFuture, FutureExt};
use tokio_postgres_rustls::MakeRustlsConnect;

type DbPool = Pool<AsyncPgConnection>;

/// Conexiones abiertas como mucho. Sin límite, deadpool usa 4 por núcleo, y en
/// una máquina compartida (Render) eso puede pasar del máximo de la base de
/// datos gratuita. Se cambia con `DATABASE_POOL_SIZE`.
const DEFAULT_POOL_SIZE: usize = 10;

/// El pool de la app, con TLS según la URL. Las conexiones se abren al usarse,
/// así que aquí solo falla una configuración imposible.
pub fn build_pool(database_url: &str) -> Result<DbPool, String> {
    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(establish_with_tls);
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(database_url, config);
    let size = env::var("DATABASE_POOL_SIZE").ok().and_then(|v| v.trim().parse().ok()).unwrap_or(DEFAULT_POOL_SIZE);
    Pool::builder(manager).max_size(size).build().map_err(|e| e.to_string())
}

fn establish_with_tls(url: &str) -> BoxFuture<'_, ConnectionResult<AsyncPgConnection>> {
    async move {
        let tls = MakeRustlsConnect::new(tls_config().clone());
        let (client, connection) = tokio_postgres::connect(url, tls)
            .await
            .map_err(|e| ConnectionError::BadConnection(error_with_causes(&e)))?;
        AsyncPgConnection::try_from_client_and_connection(client, connection).await
    }
    .boxed()
}

/// El texto de un error de tokio-postgres solo dice el tipo ("error performing
/// TLS handshake"); el motivo ("server does not support TLS", el certificado
/// que no vale...) va en las causas. Sin él no se sabe qué arreglar.
/// Estos errores no llevan la URL ni la contraseña.
fn error_with_causes(error: &dyn std::error::Error) -> String {
    let mut text = error.to_string();
    let mut cause = error.source();
    while let Some(c) = cause {
        text.push_str(": ");
        text.push_str(&c.to_string());
        cause = c.source();
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Outer(std::io::Error);
    impl std::fmt::Display for Outer {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("error performing TLS handshake")
        }
    }
    impl std::error::Error for Outer {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&self.0)
        }
    }

    #[test]
    fn error_text_includes_the_cause() {
        let e = Outer(std::io::Error::other("server does not support TLS"));
        assert_eq!(error_with_causes(&e), "error performing TLS handshake: server does not support TLS");
    }
}

/// Configuración de rustls, creada una vez. El proveedor criptográfico va
/// explícito (aws-lc-rs, el mismo que ya usa el SDK de AWS): con varios en el
/// árbol de dependencias, rustls no elige solo.
fn tls_config() -> &'static rustls::ClientConfig {
    static CONFIG: OnceLock<rustls::ClientConfig> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let roots = rustls::RootCertStore { roots: webpki_roots::TLS_SERVER_ROOTS.to_vec() };
        rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::aws_lc_rs::default_provider()))
            .with_safe_default_protocol_versions()
            .expect("aws-lc-rs supports the default TLS versions")
            .with_root_certificates(roots)
            .with_no_client_auth()
    })
}

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
