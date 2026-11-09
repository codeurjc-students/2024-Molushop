//! Logs de la aplicación con `tracing`.
//!
//! - `RUST_LOG` decide qué se ve (sintaxis de `EnvFilter`, p. ej.
//!   `warn,Molushop=debug`). Sin ella: `debug` de la app en desarrollo e `info`
//!   en release; de las dependencias, solo `warn` y el arranque del servidor.
//! - `LOG_FORMAT=json` saca una línea JSON por evento (producción); cualquier
//!   otro valor, o ninguno, texto legible.
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

#[cfg(debug_assertions)]
const DEFAULT_FILTER: &str = "warn,Molushop=debug,actix_server=info";
#[cfg(not(debug_assertions))]
const DEFAULT_FILTER: &str = "warn,Molushop=info,actix_server=info";

/// Arranca el registro de logs. Se llama una vez, al principio de `main`
/// (después de cargar el `.env`). El guard tiene que vivir hasta el final de
/// `main`: al soltarlo se escribe lo que quede pendiente (antes de un
/// `process::exit`, soltarlo a mano con `drop`).
#[must_use]
pub fn init() -> WorkerGuard {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));
    let (writer, guard) = tracing_appender::non_blocking(std::io::stdout());
    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(writer)
        // Colores solo en una terminal: en un fichero saldrían los códigos ANSI.
        .with_ansi(std::io::stdout().is_terminal());

    if env::var("LOG_FORMAT").is_ok_and(|v| v.eq_ignore_ascii_case("json")) {
        builder.json().init();
    } else {
        builder.init();
    }
    guard
}
