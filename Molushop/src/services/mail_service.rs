//! Envío de emails. `MAIL_PROVIDER` elige cómo sale:
//!
//! - `log` (o sin poner): se escribe en la consola del servidor y no sale
//!   nada. Solo para desarrollar; en `prod` está prohibido.
//! - `brevo`: por la API HTTPS de Brevo (`MAIL_API_KEY`, solo con permiso de
//!   "Transactional email"). Por API y no por SMTP porque Render, en el plan
//!   gratis, bloquea los puertos SMTP (25, 465 y 587).
//!
//! `MAIL_FROM` es el remitente, "Molushop <tienda@ejemplo.com>", y tiene que
//! estar verificado en el proveedor. `MAIL_API_KEY` y `MAIL_FROM` valen para
//! cualquier proveedor por API: para añadir otro (SES, Resend...), su rama en
//! `send` y su nombre en `KNOWN_PROVIDERS`.
//!
//! Quien llama no cambia: pide `send_password_reset` (o el email que toque) y
//! no sabe cómo sale.

use std::env;
use std::sync::OnceLock;
use std::time::Duration;

use crate::app_env::AppEnv;
use crate::i18n::Lang;

/// Un email ya redactado.
#[derive(Debug, Clone)]
pub struct Mail {
    pub to: String,
    pub subject: String,
    /// Texto plano. Si un día hace falta HTML, se añade otro campo.
    pub body: String,
}

/// Los errores no llevan ni el destinatario ni el cuerpo (que lleva el token):
/// quien llama los escribe en el log.
#[derive(Debug)]
pub enum MailError {
    /// `MAIL_PROVIDER` tiene un valor que todavía no está programado.
    UnknownProvider(String),
    /// Falta `MAIL_API_KEY` o `MAIL_FROM` no es válido (lo evita `check_mail_config`).
    Config(String),
    /// No se pudo hablar con el proveedor (red, tiempo de espera...).
    Http(String),
    /// El proveedor contestó con error: estado HTTP y su respuesta.
    Rejected { status: u16, body: String },
}

/// Proveedores programados en `send`. Al añadir otro, ponerlo aquí.
const KNOWN_PROVIDERS: &[&str] = &["log", "brevo"];

/// API de envío de Brevo (correo transaccional).
const BREVO_SEND_URL: &str = "https://api.brevo.com/v3/smtp/email";

/// Comprueba la configuración del correo antes de arrancar el servidor.
pub fn check_mail_config(app_env: AppEnv) -> Result<(), String> {
    validate_mail_config(
        env::var("MAIL_PROVIDER").ok().as_deref(),
        env::var("PUBLIC_BASE_URL").ok().as_deref(),
        env::var("MAIL_API_KEY").ok().as_deref(),
        env::var("MAIL_FROM").ok().as_deref(),
        app_env,
    )
}

/// En `prod`, `log` (o nada) está prohibido: escribiría en la consola el
/// enlace para cambiar la contraseña de cualquiera. Fuera de `dev`, sin
/// `PUBLIC_BASE_URL` los enlaces de los emails apuntarían a 127.0.0.1.
/// Un proveedor de verdad necesita `MAIL_API_KEY` y un `MAIL_FROM` válido.
fn validate_mail_config(
    provider: Option<&str>,
    base_url: Option<&str>,
    api_key: Option<&str>,
    from: Option<&str>,
    app_env: AppEnv,
) -> Result<(), String> {
    let provider = provider.map(str::trim).filter(|p| !p.is_empty());
    if let Some(p) = provider {
        if !KNOWN_PROVIDERS.contains(&p) {
            return Err(format!("MAIL_PROVIDER \"{}\" is not implemented (known: {})", p, KNOWN_PROVIDERS.join(", ")));
        }
    }
    if !app_env.allows_log_mailbox() && matches!(provider, None | Some("log")) {
        return Err(format!("MAIL_PROVIDER must be a real provider with APP_ENV={}, not \"log\"", app_env));
    }
    if app_env.requires_public_base_url() && base_url.map(str::trim).unwrap_or("").is_empty() {
        return Err(format!("PUBLIC_BASE_URL must be set with APP_ENV={}", app_env));
    }
    if let Some(p) = provider.filter(|p| *p != "log") {
        if api_key.map(str::trim).unwrap_or("").is_empty() {
            return Err(format!("MAIL_API_KEY must be set with MAIL_PROVIDER={}", p));
        }
        if parse_sender(from.unwrap_or("")).is_none() {
            return Err(format!("MAIL_FROM must be like \"Molushop <shop@example.com>\" with MAIL_PROVIDER={}", p));
        }
    }
    Ok(())
}

/// Remitente separado en nombre y dirección, como lo piden las APIs.
#[derive(Debug, PartialEq)]
struct Sender {
    name: Option<String>,
    email: String,
}

/// "Molushop <a@b.com>", "Molushop<a@b.com>", "<a@b.com>" o "a@b.com".
fn parse_sender(raw: &str) -> Option<Sender> {
    let raw = raw.trim().trim_matches('"').trim();
    let (name, email) = match (raw.find('<'), raw.rfind('>')) {
        (Some(open), Some(close)) if open < close && raw[close + 1..].trim().is_empty() => {
            let name = raw[..open].trim().trim_matches('"').trim();
            (Some(name).filter(|n| !n.is_empty()), raw[open + 1..close].trim())
        }
        (None, None) => (None, raw),
        _ => return None,
    };
    // Comprobación mínima: algo@algo.algo, sin espacios.
    let (user, domain) = email.split_once('@')?;
    let valid = !user.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
        && !email.contains(char::is_whitespace) && !domain.contains('@');
    valid.then(|| Sender { name: name.map(str::to_string), email: email.to_string() })
}

/// Un solo cliente HTTPS para toda la app (reutiliza conexiones).
fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("the HTTP client for emails could not be built")
    })
}

/// Cuerpo JSON de la API de Brevo (`POST /v3/smtp/email`).
fn brevo_payload(mail: &Mail, sender: &Sender) -> serde_json::Value {
    let mut from = serde_json::json!({ "email": sender.email });
    if let Some(name) = &sender.name {
        from["name"] = name.clone().into();
    }
    serde_json::json!({
        "sender": from,
        "to": [{ "email": mail.to }],
        "subject": mail.subject,
        "textContent": mail.body,
    })
}

async fn send_with_brevo(mail: &Mail) -> Result<(), MailError> {
    let api_key = env::var("MAIL_API_KEY").map_err(|_| MailError::Config("MAIL_API_KEY is not set".into()))?;
    let sender = parse_sender(&env::var("MAIL_FROM").unwrap_or_default())
        .ok_or_else(|| MailError::Config("MAIL_FROM is not valid".into()))?;
    let response = http_client()
        .post(BREVO_SEND_URL)
        .header("api-key", api_key.trim())
        .header("accept", "application/json")
        .json(&brevo_payload(mail, &sender))
        .send()
        .await
        .map_err(|e| MailError::Http(e.to_string()))?;
    let status = response.status();
    if status.is_success() {
        return Ok(());
    }
    // La respuesta de error de Brevo es un JSON corto ({"code", "message"}) sin datos del email.
    let mut body = response.text().await.unwrap_or_default();
    body.truncate(300);
    Err(MailError::Rejected { status: status.as_u16(), body })
}

/// Manda el email por el proveedor configurado.
pub async fn send(mail: &Mail) -> Result<(), MailError> {
    let provider = env::var("MAIL_PROVIDER").unwrap_or_else(|_| "log".to_string());
    match provider.trim() {
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
        "brevo" => send_with_brevo(mail).await,
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

    const URL: Option<&str> = Some("https://molushop.example");
    const KEY: Option<&str> = Some("xkeysib-test");
    const FROM: Option<&str> = Some("Molushop <shop@example.com>");

    #[test]
    fn development_allows_the_log_mailbox_without_public_url() {
        assert!(validate_mail_config(None, None, None, None, AppEnv::Dev).is_ok());
        assert!(validate_mail_config(Some("log"), None, None, None, AppEnv::Dev).is_ok());
    }

    #[test]
    fn pre_allows_the_log_mailbox_but_needs_public_url() {
        assert!(validate_mail_config(Some("log"), URL, None, None, AppEnv::Pre).is_ok());
        assert!(validate_mail_config(Some("log"), None, None, None, AppEnv::Pre).is_err());
    }

    #[test]
    fn prod_refuses_log_or_missing_provider() {
        assert!(validate_mail_config(None, URL, KEY, FROM, AppEnv::Prod).is_err());
        assert!(validate_mail_config(Some("log"), URL, KEY, FROM, AppEnv::Prod).is_err());
    }

    #[test]
    fn unknown_provider_is_refused_everywhere() {
        assert!(validate_mail_config(Some("smtpp"), None, KEY, FROM, AppEnv::Dev).is_err());
        assert!(validate_mail_config(Some("smtpp"), URL, KEY, FROM, AppEnv::Prod).is_err());
    }

    #[test]
    fn brevo_needs_api_key_and_valid_sender() {
        assert!(validate_mail_config(Some("brevo"), URL, KEY, FROM, AppEnv::Prod).is_ok());
        assert!(validate_mail_config(Some("brevo"), None, KEY, FROM, AppEnv::Dev).is_ok());
        assert!(validate_mail_config(Some("brevo"), URL, None, FROM, AppEnv::Prod).is_err());
        assert!(validate_mail_config(Some("brevo"), URL, Some("  "), FROM, AppEnv::Prod).is_err());
        assert!(validate_mail_config(Some("brevo"), URL, KEY, None, AppEnv::Prod).is_err());
        assert!(validate_mail_config(Some("brevo"), URL, KEY, Some("Molushop"), AppEnv::Prod).is_err());
    }

    #[test]
    fn sender_is_parsed_with_or_without_name() {
        let named = Some(Sender { name: Some("Molushop".into()), email: "shop@example.com".into() });
        assert_eq!(parse_sender("Molushop <shop@example.com>"), named);
        assert_eq!(parse_sender("Molushop<shop@example.com>"), named);
        assert_eq!(parse_sender("\"Molushop\" <shop@example.com>"), named);
        assert_eq!(parse_sender("  shop@example.com "), Some(Sender { name: None, email: "shop@example.com".into() }));
        assert_eq!(parse_sender("<shop@example.com>"), Some(Sender { name: None, email: "shop@example.com".into() }));
    }

    #[test]
    fn invalid_senders_are_refused() {
        for raw in ["", "Molushop", "Molushop <shop>", "Molushop <shop@example.com", "a <@b.com>", "x@y", "Molushop <a@b.com> extra", "a b@c.com"] {
            assert_eq!(parse_sender(raw), None, "{raw}");
        }
    }

    #[test]
    fn brevo_payload_has_sender_recipient_and_text() {
        let mail = Mail { to: "ana@example.com".into(), subject: "Hola".into(), body: "Texto".into() };
        let sender = parse_sender("Molushop <shop@example.com>").unwrap();
        assert_eq!(brevo_payload(&mail, &sender), serde_json::json!({
            "sender": { "name": "Molushop", "email": "shop@example.com" },
            "to": [{ "email": "ana@example.com" }],
            "subject": "Hola",
            "textContent": "Texto",
        }));
        let no_name = parse_sender("shop@example.com").unwrap();
        assert_eq!(brevo_payload(&mail, &no_name)["sender"], serde_json::json!({ "email": "shop@example.com" }));
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
