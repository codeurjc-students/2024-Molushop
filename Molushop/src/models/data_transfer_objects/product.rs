//aqui estará la los modelos de la página de create produc
use serde::{Deserialize, Serialize};
use crate::models::models_x;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoutesProductPanelGroup{
    pub delete_product: &'static str,
    //images
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Product{
    pub variations: Vec<ProductVariation>,
    pub name: String,
    pub status: i16,
    pub code: String,
    pub variations_titles: Vec<String>,
    pub categories: Vec<String>,
    pub id:Uuid
    //images
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductVariation{
    //pub price: f64,
    pub stock: i32,
    pub status: i16,
    pub variation: Vec<Variation>
    //images
}

impl ProductVariation{
    pub fn from(otro:&models_x::ProductVariation)->Self{
        let vars: Vec<Variation> = match &otro.attributes {
            Some(vars_xtra) => serde_json::from_value(vars_xtra.clone()).unwrap(),
            None => Vec::new()
        };
        //let vars:Vec<Variation> =serde_json::from_value(otro.attributes).unwrap(); 
        ProductVariation{
            //price: 0,
            stock: otro.stock,
            status: otro.status,
            variation: vars
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Variation{
    pub name: String,
    pub value: String,
}
/*
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Variations {
    pub variations: Vec<ProductVariation>,
} 

impl IntoIterator for Variations {
    type Item = Variation;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
*/