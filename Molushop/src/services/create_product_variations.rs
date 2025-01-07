use crate::models::models_x::ProductForm;
use crate::models::product_variation::{ProductVariation,VariationValue};
//use crate::schema::product_variations::attributes;
use uuid::Uuid;
//use serde_json::Value;
use crate::services::servicesX::{insert_product_variation,insert_product_variations};
use diesel::result::Error;

pub fn create_product_variations(product: ProductForm, product_id: Uuid) -> Result<usize,Error> {
    let variations = product.variations;
    match variations{
        Some(variations) => {
            let vec_attributes: Vec<ProductVariation> = serde_json::from_value::<Vec<ProductVariation>>(variations).unwrap_or_default();
            
            let all_combinations = get_all_combinations(&vec_attributes);
            
            insert_product_variations(&product_id,all_combinations)
        },
        None => {
            insert_product_variation(&product_id)
            
        }
    }
    //let variations_stuct = serde_json::from_value::<Vec<ProductVariation>>(variations).unwrap;
}
/*
fn combinar_listas<T: Clone>(lista1: &[T], lista2: &[T]) -> Vec<T> {
    let mut resultado:Vec<T> = Vec::with_capacity(lista1.len() + lista2.len());
    resultado.extend(lista1.iter().cloned());
    resultado.extend(lista2.iter().cloned());
    resultado
}

fn clone_and_add<T: Clone>(lista: &[T], elemento: T) -> Vec<T> {
    let mut nueva_lista = Vec::with_capacity(lista.len() + 1);
    nueva_lista.extend(lista.iter().cloned());
    nueva_lista.push(elemento);
    nueva_lista
    }
*/
fn generate_combinations(
    variations: &[ProductVariation],
    current_combination: &mut Vec<VariationValue>,
    all_combinations: &mut Vec<Vec<VariationValue>>,
    index: usize,
) {
    if index == variations.len() {
        all_combinations.push(current_combination.clone());
        return;
    }

    let variation = &variations[index];
    for value in &variation.values {
        current_combination.push(VariationValue {
            name: variation.name.clone(),
            value: value.value.clone(),
        });
        generate_combinations(variations, current_combination, all_combinations, index + 1);
        current_combination.pop();
    }
}

pub fn get_all_combinations(variations: &[ProductVariation]) -> Vec<Vec<VariationValue>> {
    let mut all_combinations: Vec<Vec<VariationValue>> = Vec::new();
    let mut current_combination: Vec<VariationValue> = Vec::new();
    generate_combinations(variations, &mut current_combination, &mut all_combinations, 0);
    all_combinations
}