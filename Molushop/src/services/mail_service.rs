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

/// Manda el email por el proveedor configurado.
pub async fn send(mail: &Mail) -> Result<(), MailError> {
    let provider = env::var("MAIL_PROVIDER").unwrap_or_else(|_| "log".to_string());
    match provider.as_str() {
        // Sin proveedor: a la consola. No sale ningún email de verdad.
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

/// El email de "He olvidado mi contraseña".
pub async fn send_password_reset(to: &str, username: &str, link: &str, minutes: i64) -> Result<(), MailError> {
    send(&Mail {
        to: to.to_string(),
        subject: "Molushop: cambia tu contraseña".to_string(),
        body: format!(
            "Hola, {}:\n\n\
             Alguien (esperamos que tú) ha pedido cambiar la contraseña de tu cuenta de Molushop.\n\
             Para elegir una nueva, abre este enlace (caduca en {} minutos y solo sirve una vez):\n\n\
             {}\n\n\
             Si no lo has pedido tú, ignora este email: tu contraseña no cambia.\n",
            username, minutes, link
        ),
    })
    .await
}
