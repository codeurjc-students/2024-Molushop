use serde::{Deserialize, Serialize};
use crate::models::models_x;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Images{
    pub images: Vec<String>,
}