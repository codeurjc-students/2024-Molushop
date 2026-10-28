use crate::models::models_x::ProductForm;
use crate::models::product_variation::{ProductVariation,VariationValue};
//use crate::schema::product_variations::attributes;
use uuid::Uuid;
//use serde_json::Value;
use crate::services::servicesX::{insert_product_variation, insert_product_variations, insert_prices_variations};
use diesel::result::Error;

use diesel_async::pg::AsyncPgConnection;

/// Crea las variaciones del producto y sus precios. Recibe la conexión de la
/// transacción de `insert_new_product_complete`: si falla el precio, las
/// variaciones tampoco quedan (antes quedaban sin fila en `prices`).
pub async fn create_product_variations(product: &ProductForm, product_id: Uuid, conn: &mut AsyncPgConnection) -> Result<usize,Error> {
    match product.variations.clone(){
        Some(variations) => {
            let vec_attributes: Vec<ProductVariation> = serde_json::from_value::<Vec<ProductVariation>>(variations).unwrap_or_default();

            let all_combinations = get_all_combinations(&vec_attributes);

            let variations_ids= insert_product_variations(&product_id,all_combinations,conn).await?;

            let number= insert_prices_variations(&variations_ids,&product.price,"EUR",conn).await?;
            Ok(number)
        },
        None => {
            let variation =  vec![insert_product_variation(&product_id,conn).await?];
            let number= insert_prices_variations(&variation,&product.price,"EUR",conn).await?;
            Ok(number)

        }
    }
    //let variations_stuct = serde_json::from_value::<Vec<ProductVariation>>(variations).unwrap;
}
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