use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug)]
pub struct BaseSpecs{
    pub specs: Vec<String>
}