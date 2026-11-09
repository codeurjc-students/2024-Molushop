//! En qué entorno corre la app: variable `APP_ENV` (`dev`, `pre` o `prod`).
//!
//! La misma imagen compilada en release sirve para `pre` y `prod`; solo
//! cambian las variables. Lo que decide cada entorno:
//!
//! | `APP_ENV` | Buzón `MAIL_PROVIDER=log` | `PUBLIC_BASE_URL` | Logs por defecto   |
//! |-----------|---------------------------|-------------------|--------------------|
//! | `dev`     | sí                        | no hace falta     | texto, app `debug` |
//! | `pre`     | sí                        | obligatoria       | JSON, app `info`   |
//! | `prod`    | no                        | obligatoria       | JSON, app `info`   |
//!
//! - Sin `APP_ENV`: `prod` en release y `dev` en una compilación de desarrollo.
//!   Olvidarla deja el modo más estricto, nunca el más permisivo.
//! - Una compilación release no acepta `dev`: no arranca.
//! - `RUST_LOG` y `LOG_FORMAT`, si están puestas, ganan a lo de la tabla.
//! - `/ui` y `/sandbox` NO dependen de esto sino de la compilación
//!   (`cfg!(debug_assertions)`): en una imagen release ese código no existe, y
//!   una variable mal puesta no puede publicarlo.

use std::env;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEnv {
    Dev,
    Pre,
    Prod,
}

impl AppEnv {
    /// Lee `APP_ENV`. Un valor desconocido, o `dev` en release, es un error:
    /// `main` no arranca.
    pub fn from_env() -> Result<AppEnv, String> {
        parse(env::var("APP_ENV").ok().as_deref(), is_release_build())
    }

    /// El de sin `APP_ENV`. Sirve también para arrancar los logs cuando
    /// `APP_ENV` es inválida, y así poder escribir el error.
    pub fn default_for_build() -> AppEnv {
        if is_release_build() { AppEnv::Prod } else { AppEnv::Dev }
    }

    /// ¿Se puede usar el buzón de mentira (`MAIL_PROVIDER=log`)? Escribe en la
    /// consola el enlace para cambiar la contraseña: nunca en producción.
    pub fn allows_log_mailbox(self) -> bool {
        self != AppEnv::Prod
    }

    /// Fuera de desarrollo los enlaces de los emails tienen que llevar el dominio.
    pub fn requires_public_base_url(self) -> bool {
        self != AppEnv::Dev
    }

    /// Logs en JSON si no se dice otra cosa con `LOG_FORMAT`.
    pub fn json_logs_by_default(self) -> bool {
        self != AppEnv::Dev
    }

    pub fn as_str(self) -> &'static str {
        match self {
            AppEnv::Dev => "dev",
            AppEnv::Pre => "pre",
            AppEnv::Prod => "prod",
        }
    }
}

impl fmt::Display for AppEnv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

fn is_release_build() -> bool {
    cfg!(not(debug_assertions))
}

fn parse(value: Option<&str>, release: bool) -> Result<AppEnv, String> {
    let value = value.map(|v| v.trim().to_ascii_lowercase()).filter(|v| !v.is_empty());
    match value.as_deref() {
        None => Ok(if release { AppEnv::Prod } else { AppEnv::Dev }),
        Some("dev") if release => Err("APP_ENV=dev is not allowed in a release build (use pre or prod)".to_string()),
        Some("dev") => Ok(AppEnv::Dev),
        Some("pre") => Ok(AppEnv::Pre),
        Some("prod") => Ok(AppEnv::Prod),
        Some(other) => Err(format!("APP_ENV \"{}\" is not valid (use dev, pre or prod)", other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_value_depends_on_the_build() {
        assert_eq!(parse(None, false), Ok(AppEnv::Dev));
        assert_eq!(parse(None, true), Ok(AppEnv::Prod));
        assert_eq!(parse(Some("  "), true), Ok(AppEnv::Prod));
    }

    #[test]
    fn known_values_ignore_case_and_spaces() {
        assert_eq!(parse(Some(" PRE "), true), Ok(AppEnv::Pre));
        assert_eq!(parse(Some("Prod"), false), Ok(AppEnv::Prod));
        assert_eq!(parse(Some("dev"), false), Ok(AppEnv::Dev));
    }

    #[test]
    fn release_build_refuses_dev() {
        assert!(parse(Some("dev"), true).is_err());
    }

    #[test]
    fn unknown_value_is_refused() {
        assert!(parse(Some("production"), true).is_err());
        assert!(parse(Some("mock"), false).is_err());
    }

    #[test]
    fn rules_per_environment() {
        assert!(AppEnv::Dev.allows_log_mailbox() && AppEnv::Pre.allows_log_mailbox());
        assert!(!AppEnv::Prod.allows_log_mailbox());
        assert!(!AppEnv::Dev.requires_public_base_url());
        assert!(AppEnv::Pre.requires_public_base_url() && AppEnv::Prod.requires_public_base_url());
    }
}
