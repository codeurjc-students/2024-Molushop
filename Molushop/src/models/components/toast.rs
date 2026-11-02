use askama::Template;
use crate::i18n::I18n;

/// Átomo toast (`ui/styles/atoms/toast.css`): aviso pequeño abajo que se quita
/// solo. Para confirmar algo que ya se ve y se puede deshacer (guardar en
/// favoritos); los errores y las preguntas siguen siendo modales.
///
/// Va en la respuesta de htmx junto a lo que sustituye, con `oob()`: htmx lo
/// añade al contenedor `#m-toasts` de `layout1.html`.
#[derive(Template, Clone, Debug)]
#[template(path = "components/toast/templates/toast.html")]
pub struct Toast {
    pub text: String,
    /// Un enlace detrás del texto ("Ver favoritos").
    pub link: Option<ToastLink>,
    /// O un botón ("Deshacer"): lo que hace lo pone el JS del componente, que
    /// lo reconoce por su atributo `data-*`.
    pub action: Option<ToastAction>,
    /// La etiqueta de la "×" para lectores de pantalla.
    pub close: String,
}

#[derive(Clone, Debug)]
pub struct ToastLink {
    pub href: String,
    pub label: String,
}

/// El botón del toast: `<button data-{name}="{value}">{label}</button>`.
#[derive(Clone, Debug)]
pub struct ToastAction {
    pub label: String,
    /// El nombre del atributo, sin `data-` (`cart-restore`).
    pub name: &'static str,
    pub value: String,
}

impl Toast {
    /// El texto es una clave de `locales/*`; la "×", en el mismo idioma.
    pub fn translated(i18n: &I18n, key: &str) -> Self {
        Self { text: i18n.t(key), link: None, action: None, close: i18n.t("modal-close") }
    }

    pub fn with_link(mut self, href: String, label: String) -> Self {
        self.link = Some(ToastLink { href, label });
        self
    }

    pub fn with_action(mut self, label: String, name: &'static str, value: String) -> Self {
        self.action = Some(ToastAction { label, name, value });
        self
    }

    /// Para añadirlo al final de otra respuesta de htmx (`hx-swap-oob`): htmx
    /// mete lo de dentro del envoltorio en `#m-toasts` y pinta el resto donde
    /// toque.
    pub fn oob(&self) -> String {
        format!(
            "<div hx-swap-oob=\"beforeend:#m-toasts\">{}</div>",
            self.render().unwrap_or_default()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oob_wraps_the_toast_for_the_container_and_escapes_the_text() {
        let toast = Toast { text: "<b>Hola</b>".to_string(), link: None, action: None, close: "Cerrar".to_string() }
            .with_link("/favorites".to_string(), "Ver favoritos".to_string());
        let html = toast.oob();
        assert!(html.starts_with("<div hx-swap-oob=\"beforeend:#m-toasts\"><div class=\"m-toast\""));
        assert!(html.contains("&#60;b&#62;Hola&#60;/b&#62;") || html.contains("&lt;b&gt;Hola&lt;/b&gt;"));
        assert!(html.contains("href=\"/favorites\""));
        assert!(html.contains("aria-label=\"Cerrar\""));
    }

    /// El valor va escapado en el atributo: el JS lo lee ya sin escapar con dataset
    #[test]
    fn action_renders_a_button_with_its_data_attribute() {
        let toast = Toast { text: "Eliminado".to_string(), link: None, action: None, close: "Cerrar".to_string() }
            .with_action("Deshacer".to_string(), "cart-restore", "{\"quantity\":2}".to_string());
        let html = toast.render().unwrap();
        assert!(html.contains("<button type=\"button\" class=\"m-toast__link m-toast__action\" data-cart-restore=\""), "{}", html);
        assert!(!html.contains("{\"quantity\""), "{}", html);
        assert!(html.contains(">Deshacer</button>"), "{}", html);
    }
}
