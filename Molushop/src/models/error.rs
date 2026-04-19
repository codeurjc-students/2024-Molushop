use serde::{Deserialize, Serialize};

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
}