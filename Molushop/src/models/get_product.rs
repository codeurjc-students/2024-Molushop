use serde::{Deserialize, Serialize};
use serde_json::Value; 
use std::fmt;
use rinja::filters::HtmlSafe;
use crate::controllers::createProduct::product;
use crate::servicesX::{combine,combine_tail_recursive};

use super::models_x::Products;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariationValue {
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Variation {
    pub name: String,
    pub values: Vec<VariationValue>,
}

impl fmt::Display for Variation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Variation: {}, Values: {:?}", self.name, self.values)
    }
}

impl fmt::Display for VariationValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariationsForm {
    pub variations: Vec<Variation>,
} 

impl fmt::Display for VariationsForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for variation in &self.variations {
            writeln!(f, "{}", variation)?;
        }
        Ok(())
    }
}


/////////////////////////
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Variations(pub Vec<Variation>);

impl fmt::Display for Variations {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for variation in &self.0 {
            writeln!(f, "{}", variation)?;
        }
        Ok(())
    }
}

// Implementación de IntoIterator para Variations
impl IntoIterator for Variations {
    type Item = Variation;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// Implementación de IntoIterator para &Variations
impl<'a> IntoIterator for &'a Variations {
    type Item = &'a Variation;
    type IntoIter = std::slice::Iter<'a, Variation>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

// Implementación de IntoIterator para &mut Variations
impl<'a> IntoIterator for &'a mut Variations {
    type Item = &'a mut Variation;
    type IntoIter = std::slice::IterMut<'a, Variation>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}
////////////////
#[derive(Serialize,Deserialize, Debug, Clone)]
pub struct GetProductForm{
    pub code : String,
    pub name: String,
    pub id: uuid::Uuid,
    pub description: String,
    pub brand: String,
    pub specs: Value,
    pub variations: Variations,
    //pub variations: VariationsForm,
    pub all_variations: Vec<Vec<String>>,
    pub images: Value,
    pub has_variations: bool,
}

impl GetProductForm{
    pub fn new_from_product(product: Products) -> Self {
        let variations = product.variations.unwrap_or(Value::Null);
        let variations = serde_json::from_value::<Variations>(variations).unwrap_or(Variations(vec![]));
        //let variations: Vec<Variation> = serde_json::from_value::<Vec<Variation>>(variations).unwrap_or(vec![]);
        //let has_variations = !variations.is_empty();
        //let variations_form = VariationsForm {
         //   variations
        //};
        let has_variations = !variations.0.is_empty();
        let mut all_variations_x:Vec<Vec<String>> = Vec::new();
        if(has_variations){
            for variation in &variations{
                let mut aux_values:Vec<String> = Vec::new();
                for value in &variation.values{
                    aux_values.push(value.value.clone());
                }
                all_variations_x.push(aux_values);
            }
        }
        let all_variations = combine_tail_recursive(all_variations_x);


        
        
        GetProductForm {
            code: product.code,
            name: product.name,
            id: product.id,
            description: product.description,
            brand: product.brand,
            specs: product.specs,
            variations,
            all_variations,
            //variations: variations_form,
            images: product.images.unwrap_or(Value::Null),
            has_variations
        }
    }
}

impl fmt::Display for GetProductForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GetProductForm {{ code: {}, name: {}, id: {}, description: {}, brand: {}, specs: {}, variations: {:?}, images: {} }}",
            self.code,
            self.name,
            self.id,
            self.description,
            self.brand,
            self.specs,
            self.variations,
            self.images
        )
    }
}