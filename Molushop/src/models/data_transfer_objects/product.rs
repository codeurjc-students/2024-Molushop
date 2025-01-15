//aqui estará la los modelos de la página de create produc
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Product{
    pub variations: Vec<ProductVariation>,
    pub name: String,
    pub status: bool,
    pub code: String,
    pub variations_titles: Vec<String>,
    //images

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductVariation{
    pub name: String,
    pub price: f64,
    pub stock: i32,
    pub status: bool,
    pub variation: Vec<Variation>
    //images
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Variation{
    pub name: String,
    pub value: String,
}
