use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Variation not found")]
    VariationNotFound,
    #[error("Not enough stock")]
    NotEnoughStock,
    #[error("Cart item not found")]
    CartItemNotFound,

    // --- Opiniones ---
    /// La estrella no está entre 1 y 5. La plantilla solo pinta cinco radios,
    /// así que esto solo salta con una petición hecha a mano.
    #[error("Invalid rating")]
    InvalidRating,
    /// El comentario pasa del tope que también impone el CHECK de la tabla.
    #[error("Comment too long")]
    CommentTooLong,
    /// Opinar sobre un producto que no existe: lo canta la FK de `ratings`.
    #[error("Product not found")]
    ProductNotFound,
    /// Opinar sin haber comprado el producto. La plantilla ya no pinta el
    /// formulario en ese caso; esto es lo que lo impide de verdad.
    #[error("Product not purchased")]
    NotPurchased,

    // --- Checkout ---
    /// No queda ninguna línea que comprar: el carrito está vacío o el `lines`
    /// del formulario no casa con nada del usuario.
    #[error("Empty cart")]
    EmptyCart,
    /// Una línea no tiene precio o moneda. El pedido se rechaza entero: es
    /// preferible a cobrar 0 por algo.
    #[error("Missing price")]
    MissingPrice,
    /// Stock insuficiente en UNA línea concreta, detectado ya dentro de la
    /// transacción. Lleva consigo qué línea y cuánto queda porque el checkout
    /// lo enseña en pantalla: es el fallo realista (alguien compró mientras el
    /// usuario rellenaba la dirección).
    #[error("Not enough stock for line {line_id}")]
    NotEnoughStockLine {
        line_id: Uuid,
        product_name: String,
        available: i32,
    },
}

/// `AsyncConnection::transaction` exige que el error de la transacción sepa
/// construirse desde el de diesel, porque es él quien decide hacer ROLLBACK.
/// Sin esto no se puede usar `ServiceError` dentro de una transacción.
impl From<diesel::result::Error> for ServiceError {
    fn from(e: diesel::result::Error) -> Self {
        ServiceError::InternalServerError(e.to_string())
    }
}
