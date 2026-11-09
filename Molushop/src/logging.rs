//! Logs de la aplicación con `tracing`.
//!
//! - `RUST_LOG` decide qué se ve (sintaxis de `EnvFilter`, p. ej.
//!   `warn,Molushop=debug`). Sin ella: `debug` de la app con `APP_ENV=dev` e
//!   `info` en `pre` y `prod`; de las dependencias, solo `warn` y el arranque
//!   del servidor.
//! - `LOG_FORMAT=json` saca una línea JSON por evento y cualquier otro valor,
//!   texto legible. Sin ella: texto en `dev`, JSON en `pre` y `prod`.
//!
//! Los `log::` de las dependencias (actix) también pasan por aquí.
//!
//! Salen por stdout, que es lo que recogen Docker y Render. Se escriben desde
//! un hilo aparte (`tracing_appender::non_blocking`): una petición nunca espera
//! a que la consola trague. Si se acumulan más de 128.000 líneas sin escribir,
//! las nuevas se descartan antes que frenar el servidor.
//! Nunca se registran contraseñas, tokens ni cookies.

use std::env;
use std::io::IsTerminal;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

use crate::app_env::AppEnv;

const DEV_FILTER: &str = "warn,Molushop=debug,actix_server=info";
const DEPLOYED_FILTER: &str = "warn,Molushop=info,actix_server=info";

/// Arranca el registro de logs. Se llama una vez, al principio de `main`
/// (después de cargar el `.env`). El guard tiene que vivir hasta el final de
/// `main`: al soltarlo se escribe lo que quede pendiente (antes de un
/// `process::exit`, soltarlo a mano con `drop`).
#[must_use]
pub fn init(app_env: AppEnv) -> WorkerGuard {
    let default_filter = if app_env == AppEnv::Dev { DEV_FILTER } else { DEPLOYED_FILTER };
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));
    let (writer, guard) = tracing_appender::non_blocking(std::io::stdout());
    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(writer)
        // Colores solo en una terminal: en un fichero saldrían los códigos ANSI.
        .with_ansi(std::io::stdout().is_terminal());

    let json = match env::var("LOG_FORMAT") {
        Ok(format) => format.eq_ignore_ascii_case("json"),
        Err(_) => app_env.json_logs_by_default(),
    };
    if json {
        builder.json().init();
    } else {
        builder.init();
    }
    guard
}
