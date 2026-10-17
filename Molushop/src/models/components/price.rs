use askama::Template;
use bigdecimal::BigDecimal;

use crate::constants::money::{format_amount, symbol_of};

/// Átomo: precio. Pinta un importe con su moneda en el formato de la tienda
/// ("9,00 €"). El formato está en `constants::money::format_amount`, y el JS
/// que recalcula precios usa `MONEY.format()` (ui/utils/base.js), que da el
/// mismo texto.
///
/// Igual que `Stars`: cada modelo tiene un método que devuelve el `Price` y la
/// plantilla solo lo escribe.
///
///     pub fn price_tag(&self) -> Price { Price::new(&self.price, &self.currency) }
///     {{ product.price_tag() }}
///
/// (`price_tag` y no `price`, que suele ser ya el nombre del campo.)
///
/// El CSS es `ui/styles/atoms/price.css`.
#[derive(Template, Clone, Debug)]
#[template(path = "components/price/templates/price.html")]
pub struct Price {
    /// `None` = no hay precio: pinta "No disponible" y esconde la moneda.
    pub amount: Option<BigDecimal>,
    /// Código ISO ("EUR"), como en la BD; el símbolo sale al pintar.
    pub currency: String,
    pub large: bool,
}

impl Price {
    pub fn new(amount: &BigDecimal, currency: &str) -> Self {
        Price { amount: Some(amount.clone()), currency: currency.to_string(), large: false }
    }

    /// Para un precio que puede faltar (una variación sin fila en `prices`).
    pub fn maybe(amount: Option<&BigDecimal>, currency: &str) -> Self {
        Price { amount: amount.cloned(), currency: currency.to_string(), large: false }
    }

    /// El precio grande de la cabecera de la ficha.
    pub fn lg(mut self) -> Self {
        self.large = true;
        self
    }

    pub fn has_amount(&self) -> bool {
        self.amount.is_some()
    }

    /// "9,00", o "No disponible" si no hay precio.
    pub fn amount_label(&self) -> String {
        match &self.amount {
            Some(amount) => format_amount(amount),
            None => "No disponible".to_string(),
        }
    }

    pub fn symbol(&self) -> String {
        symbol_of(&self.currency)
    }
}

/// Lo que pinta ya es HTML: sin `|safe` en cada llamada. El importe y el
/// símbolo los escapa la propia plantilla.
impl askama::filters::HtmlSafe for Price {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn renders_amount_and_symbol() {
        let price = Price::new(&BigDecimal::from_str("12345.6").unwrap(), "EUR");
        let html = price.render().unwrap();
        assert!(html.contains(">12.345,60<"), "{}", html);
        assert!(html.contains("&nbsp;€"), "{}", html);
        assert!(!html.contains("m-price--lg"));
    }

    #[test]
    fn missing_price_hides_currency() {
        let html = Price::maybe(None, "EUR").lg().render().unwrap();
        assert!(html.contains("No disponible"), "{}", html);
        assert!(html.contains("m-price--lg"));
        assert!(html.contains(" hidden"), "{}", html);
    }
}
