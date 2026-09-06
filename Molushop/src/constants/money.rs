
/// Moneda por defecto cuando un precio no la trae. Es un código ISO, igual que
/// `prices.currency`, `orders.currency` y `order_items.currency`: en la BD y en
/// los modelos SIEMPRE viaja el código, nunca el símbolo. El símbolo es cosa de
/// la plantilla (`CartItemData::currency_symbol()`).
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
