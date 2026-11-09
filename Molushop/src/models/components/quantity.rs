use askama::Template;
use crate::i18n::{I18n, Lang};

/// Átomo: selector de cantidad (− 1 +). Pinta los dos botones y el input; el
/// JS común (`QUANTITY` en ui/utils/base.js) suma o resta sin pasarse de los
/// límites del input y avisa con el evento `m-qty-change`. Qué hacer con la
/// nueva cantidad (nada hasta "Añadir al carrito", o mandarla al servidor) lo
/// decide cada componente.
///
/// Igual que `Price`: el modelo tiene un método que devuelve el `Quantity` y la
/// plantilla solo lo escribe.
///
///     pub fn quantity_tag(&self) -> Quantity { Quantity::new(self.quantity, 0, Some(self.stock)) }
///     {{ item.quantity_tag() }}
///
/// El CSS es `ui/styles/atoms/quantity.css`.
#[derive(Template, Clone, Debug)]
#[template(path = "components/quantity/templates/quantity.html")]
pub struct Quantity {
    pub value: i32,
    pub min: i32,
    /// `None` = sin tope.
    pub max: Option<i32>,
    pub small: bool,
    /// En el mínimo, el − es una papelera que avisa con `m-qty-remove` (el
    /// carrito: con 1 unidad, quitar la línea). Si no, en el mínimo se apaga.
    pub removable: bool,
    /// Las etiquetas de los botones y del campo (`localized`).
    pub lang: Lang,
}

impl Quantity {
    /// `value` se ajusta a los límites: si el stock bajó por debajo de lo que
    /// hay en el carrito, el input sale ya con el máximo.
    pub fn new(value: i32, min: i32, max: Option<i32>) -> Self {
        let max = max.map(|m| m.max(min));
        let mut value = value.max(min);
        if let Some(m) = max {
            value = value.min(m);
        }
        Quantity { value, min, max, small: false, removable: false, lang: Lang::DEFAULT }
    }

    /// El del carrito, más bajo.
    pub fn sm(mut self) -> Self {
        self.small = true;
        self
    }

    /// Con papelera en el mínimo (ver `removable`).
    pub fn removable(mut self) -> Self {
        self.removable = true;
        self
    }

    /// En el idioma de la página: `{{ item.quantity_tag().localized(i18n) }}`.
    pub fn localized(mut self, i18n: &I18n) -> Self {
        self.lang = i18n.lang;
        self
    }

    /// Una etiqueta (`qty-label`, `qty-decrease`, `qty-increase`).
    pub fn label(&self, key: &str) -> String {
        self.lang.t(key)
    }

    pub fn at_min(&self) -> bool {
        self.value <= self.min
    }

    /// El − apagado: en el mínimo y sin papelera.
    pub fn minus_disabled(&self) -> bool {
        self.at_min() && !self.removable
    }

    /// El − es ahora la papelera.
    pub fn minus_removes(&self) -> bool {
        self.at_min() && self.removable
    }

    pub fn at_max(&self) -> bool {
        self.max.is_some_and(|m| self.value >= m)
    }
}

impl askama::filters::HtmlSafe for Quantity {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_value_to_limits() {
        assert_eq!(Quantity::new(5, 1, Some(3)).value, 3);
        assert_eq!(Quantity::new(0, 1, Some(3)).value, 1);
        assert_eq!(Quantity::new(50, 0, None).value, 50);
        // Sin stock: el tope no baja del mínimo.
        assert_eq!(Quantity::new(2, 1, Some(0)).value, 1);
    }

    #[test]
    fn disables_buttons_at_limits() {
        let html = Quantity::new(1, 1, Some(3)).render().unwrap();
        assert!(html.contains("data-step=\"-1\" disabled"), "{}", html);
        assert!(!html.contains("data-step=\"1\" disabled"), "{}", html);
        assert!(html.contains("max=\"3\""), "{}", html);

        let html = Quantity::new(3, 1, Some(3)).sm().render().unwrap();
        assert!(html.contains("data-step=\"1\" disabled"), "{}", html);
        assert!(html.contains("m-qty--sm"), "{}", html);
    }

    /// En el mínimo no se apaga: es la papelera, con su etiqueta, y lleva las
    /// dos etiquetas para que el JS las cambie
    #[test]
    fn removable_turns_minus_into_trash_at_min() {
        let html = Quantity::new(1, 1, Some(3)).removable().render().unwrap();
        assert!(!html.contains("disabled"), "{}", html);
        assert!(html.contains("m-qty__btn is-remove"), "{}", html);
        assert!(html.contains("aria-label=\"Eliminar\""), "{}", html);
        assert!(html.contains("data-label-remove=\"Eliminar\""), "{}", html);

        let html = Quantity::new(2, 1, Some(3)).removable().render().unwrap();
        assert!(!html.contains("is-remove"), "{}", html);
        assert!(html.contains("aria-label=\"Quitar una unidad\""), "{}", html);
    }

    #[test]
    fn no_max_attribute_without_limit() {
        let html = Quantity::new(1, 1, None).render().unwrap();
        assert!(!html.contains("max="), "{}", html);
    }
}
