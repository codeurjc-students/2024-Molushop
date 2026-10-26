use askama::Template;
use crate::i18n::I18n;

#[derive(Template,Clone,Debug)]
#[template(path = "components/modal/templates/message.html")]
pub struct MessageModal{
    pub text: String,
    /// El texto del botón: "Aceptar", u otro idioma con `translated`.
    pub accept: String,
    /// La etiqueta de la "×" para lectores de pantalla.
    pub close: String,
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
        }
    }
    /// Un aviso de la tienda: el texto es una clave de `locales/*` y el botón
    /// va en el mismo idioma.
    pub fn translated(i18n: &I18n, key: &str) -> Self {
        Self{
            text: i18n.t(key),
            accept: i18n.t("modal-accept"),
            close: i18n.t("modal-close"),
        }
    }
}