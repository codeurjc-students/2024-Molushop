// logica de implementación
// obtener productos y variaciones
use serde::{Serialize, Deserialize};
use diesel::result::Error;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use crate::models::data_transfer_objects::product::Product;
use crate::models::models_x::{ProductVariation,Category};
use crate::models::data_transfer_objects::product;
//use crate::schema::products::variations;
use crate::services::servicesX::{get_seller_products,get_product_categories,get_product_variations};
type DbPool = Pool<AsyncPgConnection>;
use uuid::Uuid;

pub async fn get_products(user_id:&Uuid,pool: &DbPool) -> Result<Vec<Product>,Error>{

    let products = get_seller_products(user_id, pool).await?;
    let mut products_final:Vec<Product> = Vec::new();

    for product in products{
        let categories: Vec<Category> = get_product_categories(&product.id, pool).await?;
        let categories_names:Vec<String> = categories.iter().map(|c| c.name.clone()).collect();
        let vars: Vec<ProductVariation> = get_product_variations(&product.id, pool).await?;
        //let variations:Vec<product::ProductVariation> = Vec::new();
        
        let variation_titles:Vec<String> = match product.variation_titles{
            Some(titles) =>{
                let titles_aux:VariationTitles = serde_json::from_value(titles).unwrap();
                titles_aux.titulos
            },
            None => Vec::new()     
        };
        let variations: Vec<product::ProductVariation> = vars.iter().map(|v| product::ProductVariation::from(v)).collect();  

        let product_final = Product{
            variations: variations,
            name: product.name,
            status: product.status,
            code: product.code,
            variations_titles: variation_titles,
            categories: categories_names,
            id: product.id
        };
        products_final.push(product_final);
    }
        
    Ok(products_final)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariationTitles{
    pub titulos: Vec<String>,
}


// obtener los productos de un vendedor 