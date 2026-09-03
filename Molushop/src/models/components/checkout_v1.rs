use askama::Template;
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};

use crate::constants::urls::{CART_URL, CHECKOUT_CONFIRM_URL};
use crate::models::components::cart_total_v1::CartTotalData;

#[derive(Template,Clone,Debug)]
#[template(path = "components/checkout_v1/checkout_v1.html")]
pub struct CheckoutV1{
    pub checkout: CheckoutData
}

#[derive(Serialize, Debug, Clone)]
pub struct CheckoutRoutes{
    /// Endpoint que creará el pedido. TODAVÍA NO EXISTE: lo registra la parte 3.
    /// Cuando haya controlador de checkout, este ROUTES se mueve allí, que es
    /// donde el resto del proyecto construye sus rutas.
    pub confirm: String,
    pub cart: String
}

lazy_static! {
    pub static ref ROUTES: CheckoutRoutes = CheckoutRoutes{
        confirm: CHECKOUT_CONFIRM_URL.to_string(),
        cart: CART_URL.to_string()
    };
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
    pub line_ids: String
}

impl Default for CheckoutData {
    fn default() -> Self {
        Self {
            routes: &ROUTES,
            cart_total: CartTotalData::default(),
            address: AddressData::default(),
            line_ids: String::new()
        }
    }
}

impl CheckoutData {
    pub fn is_empty(&self) -> bool {
        self.cart_total.is_empty()
    }
}
