use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Debug)]
pub struct BaseSpecs{
    pub specs: Vec<String>
}