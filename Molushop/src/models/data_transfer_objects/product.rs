//aqui estará la los modelos de la página de create produc
use serde::{Deserialize, Serialize};
use crate::models::models_x;
use uuid::Uuid;
use bigdecimal::BigDecimal;
use crate::services::servicesX::get_variation_price;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoutesProductPanelGroup{
    pub delete_product: &'static str,
    pub delete_product_modal: &'static str,
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
    pub price: BigDecimal,
    pub stock: i32,
    pub status: i16,
    pub variation: Vec<Variation>
    //images
}

impl ProductVariation{
    pub fn from(otro_pv:&models_x::ProductVariation,otro_p:&models_x::Price)->Self{
        let vars: Vec<Variation> = match &otro_pv.attributes {
            Some(vars_xtra) => serde_json::from_value(vars_xtra.clone()).unwrap(),
            None => Vec::new()
        };
        
        //let vars:Vec<Variation> =serde_json::from_value(otro.attributes).unwrap(); 
        ProductVariation{
            price: otro_p.price.clone(),
            stock: otro_pv.stock,
            status: otro_pv.status,
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