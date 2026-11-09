use actix_web::http::StatusCode;
use askama::Template;
use crate::i18n::I18n;
use crate::middleware::request_log;

#[derive(Template,Clone,Debug)]
#[template(path = "components/modal/templates/message.html")]
pub struct MessageModal{
    pub text: String,
    /// El texto del botón: "Aceptar", u otro idioma con `translated`.
    pub accept: String,
    /// La etiqueta de la "×" para lectores de pantalla.
    pub close: String,
    /// Debajo del texto en los avisos de error: "Error 500 · ref. 7f3a9c2e".
    pub reference: Option<String>,
}

impl MessageModal{
    pub fn new(text: &str)-> Self{
        Self::from_string(text.to_string())
    }
    pub fn from_string(text: String)-> Self{
        Self{
            text,
            accept: "Aceptar".to_string(),
            close: "Cerrar".to_string(),
            reference: None,
        }
    }
    /// Un aviso de la tienda: el texto es una clave de `locales/*` y el botón
    /// va en el mismo idioma.
    pub fn translated(i18n: &I18n, key: &str) -> Self {
        Self{
            text: i18n.t(key),
            accept: i18n.t("modal-accept"),
            close: i18n.t("modal-close"),
            reference: None,
        }
    }
    /// Para los avisos de error: el código y el principio del id de la
    /// petición (`request_log`), para buscarla en el log. Fuera de una
    /// petición se queda sin referencia.
    pub fn with_error_ref(mut self, status: StatusCode, i18n: &I18n) -> Self {
        self.reference = request_log::current_reference()
            .map(|reference| i18n.t2("modal-error-reference", "status", status.as_str(), "reference", reference));
        self
    }
}