use askama::Template;
use serde::{Serialize, Deserialize};

use uuid::Uuid;

use crate::controllers::components::checkout_controller::ROUTES;
use crate::models::components::cart_total_v1::CartTotalData;

#[derive(Template,Clone,Debug)]
#[template(path = "components/checkout_v1/checkout_v1.html")]
pub struct CheckoutV1{
    pub checkout: CheckoutData
}

#[derive(Serialize, Debug, Clone)]
pub struct CheckoutRoutes{
    /// Endpoint que crea el pedido.
    pub confirm: String,
    pub cart: String
}

/// Dirección de envío tal como se pinta en el formulario. Los campos son los de
/// `customer_address`; `street2` es el único opcional.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AddressData{
    pub street1: String,
    pub street2: String,
    pub postal_code: String,
    pub city: String,
    pub province: String
}

/// Lo que manda el formulario de confirmación. Es un POST de HTML normal
/// (urlencoded), NO JSON: el checkout tiene que funcionar sin JS, así que el
/// handler recibe `web::Form`. Vive en el modelo y no en el controlador porque
/// lo comparten el controlador y el servicio.
/// `serde(default)` en todos los campos a propósito: sin él, un POST al que le
/// falte cualquiera (`street2` es opcional en el formulario) se queda en un 400
/// crudo de actix antes de llegar al handler. Con él, el campo llega vacío y lo
/// caza la validación de `address_is_complete`, que sí sabe repintar la página
/// con un mensaje en condiciones.
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct ConfirmForm{
    /// Ids de las líneas elegidas, separados por coma. Viaja en un hidden que
    /// rellena el carrito; vacío significa "todo el carrito".
    pub lines: String,
    pub street1: String,
    pub street2: String,
    pub postal_code: String,
    pub city: String,
    pub province: String
}

impl ConfirmForm {
    /// Lo tecleado, sin espacios sobrantes, para poder repintar el formulario
    /// tal cual si el pedido falla.
    pub fn address(&self) -> AddressData {
        AddressData {
            street1: self.street1.trim().to_string(),
            street2: self.street2.trim().to_string(),
            postal_code: self.postal_code.trim().to_string(),
            city: self.city.trim().to_string(),
            province: self.province.trim().to_string()
        }
    }

    /// Los campos obligatorios de `customer_address`. `street2` es el único que
    /// puede faltar.
    pub fn address_is_complete(&self) -> bool {
        let a = self.address();
        !a.street1.is_empty()
            && !a.postal_code.is_empty()
            && !a.city.is_empty()
            && !a.province.is_empty()
    }
}

/// Qué salió mal al confirmar, para pintarlo sobre la propia página de checkout.
/// No se redirige con un `?error=`: eso perdería la dirección recién tecleada y
/// `customer_address` está vacía, así que no habría de dónde recuperarla.
#[derive(Serialize, Debug, Clone)]
pub struct CheckoutError{
    pub message: String,
    /// Línea culpable, si el fallo es de una en concreto (stock). La plantilla
    /// la marca para que el usuario vea cuál.
    pub line_id: Option<Uuid>
}

#[derive(Serialize, Debug, Clone)]
pub struct CheckoutData{
    pub routes: &'static CheckoutRoutes,
    /// Líneas y totales, ya filtrados por lo que el usuario eligió en la cesta.
    /// Se calculan SIEMPRE en el servidor: el `?lines=` sólo dice qué líneas,
    /// nunca cuánto valen.
    pub cart_total: CartTotalData,
    /// Dirección ya guardada del usuario, si tiene alguna. Hoy siempre viene
    /// vacía: `customer_address` no tiene filas todavía.
    pub address: AddressData,
    /// Ids de las líneas elegidas, separados por coma, tal cual viajarán en el
    /// POST de confirmación
    pub line_ids: String,
    /// Sólo se rellena cuando el POST de confirmación ha fallado y se repinta
    /// la página. En el GET normal es None.
    pub error: Option<CheckoutError>
}

impl Default for CheckoutData {
    fn default() -> Self {
        Self {
            routes: &ROUTES,
            cart_total: CartTotalData::default(),
            address: AddressData::default(),
            line_ids: String::new(),
            error: None
        }
    }
}

impl CheckoutData {
    pub fn is_empty(&self) -> bool {
        self.cart_total.is_empty()
    }

    /// True si el error apunta a esta línea, para marcarla en la plantilla.
    pub fn line_failed(&self, line_id: &Uuid) -> bool {
        match &self.error {
            Some(e) => e.line_id.as_ref() == Some(line_id),
            None => false
        }
    }
}
