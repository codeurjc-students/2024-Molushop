use serde::{Deserialize, Serialize};
//use serde_json::Value; 
//use std::fmt;

//modelo de las variaciones
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariationValue {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariationValues {
    pub values: Vec<VariationValue>,
}



//modelo de las variaciones del producto
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProductVariation {
    pub name: String,
    pub values: Vec<Val>,
}

#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct Val {
    pub value: String,
}


