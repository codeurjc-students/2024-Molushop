
use bigdecimal::{BigDecimal, RoundingMode};

/// Moneda por defecto cuando un precio no la trae. Es un código ISO, igual que
/// `prices.currency`, `orders.currency` y `order_items.currency`: en la BD y en
/// los modelos SIEMPRE viaja el código, nunca el símbolo. El símbolo es cosa de
/// la plantilla (el átomo `Price`).
pub const DEFAULT_CURRENCY: &str = "EUR";

/// Traduce el código ISO a su símbolo para pintarlo. Si no lo conoce devuelve el
/// código tal cual, que es preferible a inventarse un símbolo.
///
/// Vive aquí y no en el componente del carrito porque lo usan también la ficha
/// de producto y cualquiera que tenga que enseñar un importe: es presentación,
/// no un dato de nadie en concreto.
pub fn symbol_of(code: &str) -> String {
    match code.trim() {
        "EUR" => "€".to_string(),
        "USD" => "$".to_string(),
        "GBP" => "£".to_string(),
        other if !other.is_empty() => other.to_string(),
        _ => "€".to_string()
    }
}

/// Un importe como se escribe en español: dos decimales con coma y punto de
/// miles a partir de cinco cifras ("9,00", "1234,56", "12.345,67"). Es lo mismo
/// que hace `Intl.NumberFormat("es-ES")` en el navegador, que usa
/// `MONEY.format()` (ui/utils/base.js) cuando el JS recalcula un precio: los
/// dos tienen que dar el mismo texto.
pub fn format_amount(amount: &BigDecimal) -> String {
    let rounded: String = amount.with_scale_round(2, RoundingMode::HalfUp).to_string();
    let (sign, digits) = match rounded.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", rounded.as_str()),
    };
    let (int_part, dec_part) = digits.split_once('.').unwrap_or((digits, "00"));

    let int_part = if int_part.len() >= 5 {
        let mut grouped = String::new();
        for (i, c) in int_part.chars().enumerate() {
            if i > 0 && (int_part.len() - i) % 3 == 0 {
                grouped.push('.');
            }
            grouped.push(c);
        }
        grouped
    } else {
        int_part.to_string()
    };
    // -0,00 no existe: un descuento redondeado a cero se pinta sin signo.
    let sign = if int_part.chars().all(|c| c == '0' || c == '.') && dec_part == "00" { "" } else { sign };

    format!("{}{},{}", sign, int_part, dec_part)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn fmt(s: &str) -> String {
        format_amount(&BigDecimal::from_str(s).unwrap())
    }

    #[test]
    fn two_decimals_with_comma() {
        assert_eq!(fmt("9"), "9,00");
        assert_eq!(fmt("9.5"), "9,50");
        assert_eq!(fmt("150.99"), "150,99");
        assert_eq!(fmt("0.005"), "0,01");
        assert_eq!(fmt("2.344"), "2,34");
    }

    #[test]
    fn thousands_from_five_digits() {
        assert_eq!(fmt("1234.56"), "1234,56");
        assert_eq!(fmt("12345.67"), "12.345,67");
        assert_eq!(fmt("1234567"), "1.234.567,00");
    }

    #[test]
    fn negatives() {
        assert_eq!(fmt("-5"), "-5,00");
        assert_eq!(fmt("-0.001"), "0,00");
    }
}
