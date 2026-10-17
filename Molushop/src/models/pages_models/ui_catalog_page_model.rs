use askama::Template;
// La plantilla los construye para enseñar cada átomo.
use crate::models::components::stars::Stars;
use crate::models::components::modal_1_model::Modal1;
use crate::models::components::warning_modal::WarningModal;
use crate::models::components::price::Price;
use bigdecimal::BigDecimal;
use std::str::FromStr;

/// Catálogo del sistema de diseño (`/ui`): las variables de `tokens.css` y los
/// átomos. Es una página estática; los dos campos son los que pide `layout1`.
#[derive(Template,Clone,Debug)]
#[template(path="pages/ui_catalog/ui_catalog.html")]
pub struct UiCatalogPage{
    pub user_logged: bool,
    pub page_name:String
}

impl UiCatalogPage {
    /// Precio de ejemplo para el catálogo ("150.99" -> 150,99 €).
    pub fn demo_price(&self, amount: &str) -> Price {
        Price::new(&BigDecimal::from_str(amount).unwrap_or_default(), "EUR")
    }

    pub fn demo_price_missing(&self) -> Price {
        Price::maybe(None, "EUR")
    }

    /// El modal de confirmación de ejemplo: sin htmx, el "Sí" solo lo cierra.
    pub fn demo_confirm(&self) -> WarningModal {
        WarningModal {
            method: String::new(),
            message: "¿Estás seguro de que deseas eliminar este producto?".to_string(),
            endpoint: String::new(),
            target: String::new(),
            swap: String::new(),
            hyperscript_action: String::new(),
            htmx_active: false,
        }
    }
}
