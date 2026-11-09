//! Envío de emails. Todavía no hay proveedor: el email se escribe en la
//! consola del servidor (modo "log"), que vale para desarrollar y probar.
//!
//! Para conectar uno de verdad, solo hay que tocar este fichero:
//!
//! 1. Añadir la variable `MAIL_PROVIDER` al `.env` ("smtp" o "ses") y sus datos
//!    (SMTP: servidor, usuario y contraseña; SES: la región y el remitente
//!    verificado; las credenciales de AWS ya las lee `aws-config` para S3).
//! 2. Escribir la rama de ese proveedor en `send`: SMTP con la librería
//!    `lettre`; SES con `aws-sdk-sesv2`, igual que S3 usa `aws-sdk-s3`.
//! 3. `MAIL_FROM` es el remitente ("Molushop <no-reply@tudominio>").
//!
//! Quien llama no cambia: pide `send_password_reset` (o el email que toque) y
//! no sabe cómo sale.

use std::env;

use crate::i18n::Lang;

/// Un email ya redactado.
#[derive(Debug, Clone)]
pub struct Mail {
    pub to: String,
    pub subject: String,
    /// Texto plano. Si un día hace falta HTML, se añade otro campo.
    pub body: String,
}

#[derive(Debug)]
pub enum MailError {
    /// `MAIL_PROVIDER` tiene un valor que todavía no está programado.
    UnknownProvider(String),
}

/// Proveedores programados en `send`. Al añadir "ses" o "smtp", ponerlos aquí.
const KNOWN_PROVIDERS: &[&str] = &["log"];

/// Comprueba la configuración del correo antes de arrancar el servidor.
pub fn check_mail_config() -> Result<(), String> {
    validate_mail_config(
        env::var("MAIL_PROVIDER").ok().as_deref(),
        env::var("PUBLIC_BASE_URL").ok().as_deref(),
        cfg!(not(debug_assertions)),
    )
}

/// En release, `log` (o nada) está prohibido: escribiría en la consola el
/// enlace para cambiar la contraseña de cualquiera. Y sin `PUBLIC_BASE_URL`
/// los enlaces de los emails apuntarían a 127.0.0.1.
fn validate_mail_config(provider: Option<&str>, base_url: Option<&str>, release: bool) -> Result<(), String> {
    let provider = provider.map(str::trim).filter(|p| !p.is_empty());
    if let Some(p) = provider {
        if !KNOWN_PROVIDERS.contains(&p) {
            return Err(format!("MAIL_PROVIDER \"{}\" is not implemented (known: {})", p, KNOWN_PROVIDERS.join(", ")));
        }
    }
    if release {
        match provider {
            None | Some("log") => return Err("MAIL_PROVIDER must be a real provider in release, not \"log\"".to_string()),
            _ => {}
        }
        if base_url.map(str::trim).unwrap_or("").is_empty() {
            return Err("PUBLIC_BASE_URL must be set in release".to_string());
        }
    }
    Ok(())
}

/// Manda el email por el proveedor configurado.
pub async fn send(mail: &Mail) -> Result<(), MailError> {
    let provider = env::var("MAIL_PROVIDER").unwrap_or_else(|_| "log".to_string());
    match provider.as_str() {
        // Sin proveedor: a la consola. No sale ningún email de verdad.
        // println! y no tracing a propósito: es un buzón de mentira para
        // desarrollo y el cuerpo lleva el enlace con el token, que no debe
        // acabar en el log. En producción, MAIL_PROVIDER=log está prohibido.
        "log" => {
            println!(
                "\n===== EMAIL (MAIL_PROVIDER=log, no se envía) =====\nPara: {}\nAsunto: {}\n\n{}\n==================================================\n",
                mail.to, mail.subject, mail.body
            );
            Ok(())
        }
        // Aquí irán "smtp" y "ses".
        other => Err(MailError::UnknownProvider(other.to_string())),
    }
}

/// URL pública de la tienda, para los enlaces de los emails. En producción hay
/// que ponerla en el `.env` (`PUBLIC_BASE_URL=https://tudominio`).
pub fn public_base_url() -> String {
    env::var("PUBLIC_BASE_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string())
        .trim_end_matches('/')
        .to_string()
}

/// El email de "He olvidado mi contraseña", en `lang` (el de la página desde
/// la que se pidió). Los textos están en `locales/*/account.ftl` (mail-reset-*).
pub async fn send_password_reset(to: &str, username: &str, link: &str, minutes: i64, lang: Lang) -> Result<(), MailError> {
    send(&password_reset_mail(to, username, link, minutes, lang)).await
}

fn password_reset_mail(to: &str, username: &str, link: &str, minutes: i64, lang: Lang) -> Mail {
    Mail {
        to: to.to_string(),
        subject: lang.t("mail-reset-subject"),
        body: format!("{}\n", lang.t_args("mail-reset-body", &[
            ("username", username.to_string().into()),
            ("minutes", minutes.into()),
            ("link", link.to_string().into()),
        ])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn development_allows_the_log_mailbox() {
        assert!(validate_mail_config(None, None, false).is_ok());
        assert!(validate_mail_config(Some("log"), None, false).is_ok());
    }

    #[test]
    fn release_refuses_log_or_missing_provider() {
        assert!(validate_mail_config(None, Some("https://molushop.example"), true).is_err());
        assert!(validate_mail_config(Some("log"), Some("https://molushop.example"), true).is_err());
    }

    #[test]
    fn unknown_provider_is_refused_everywhere() {
        assert!(validate_mail_config(Some("smtpp"), None, false).is_err());
        assert!(validate_mail_config(Some("smtpp"), Some("https://molushop.example"), true).is_err());
    }

    #[test]
    fn password_reset_mail_in_each_language() {
        let es = password_reset_mail("a@b.c", "ana", "http://x/password/reset?token=t", 30, Lang::Es);
        assert_eq!(es.subject, "Molushop: cambia tu contraseña");
        assert!(es.body.starts_with("Hola, ana:\n\nAlguien"), "{}", es.body);
        assert!(es.body.contains("caduca en 30 minutos") && es.body.contains("\n\nhttp://x/password/reset?token=t\n\n"), "{}", es.body);
        let en = password_reset_mail("a@b.c", "ana", "http://x/en/password/reset?token=t", 30, Lang::En);
        assert_eq!(en.subject, "Molushop: change your password");
        assert!(en.body.starts_with("Hi ana,\n\nSomeone"), "{}", en.body);
    }
}
