use serde::{Serialize, Deserialize};
use diesel::result::Error;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use crate::models::data_transfer_objects::product::Product;
use crate::models::models_x::{ProductVariation,Category};
use crate::models::data_transfer_objects::product;
//use crate::schema::products::variations;
use crate::services::servicesX::{get_product_seller,get_product_variations,get_product_with_images1};
type DbPool = Pool<AsyncPgConnection>;
use uuid::Uuid;
use crate::models::components::edit_product::{ProductEdit,ProductEdit2,ImageCategories2,General,Images,Prices,Variations,Variation,Attribute};
use crate::models::data_transfer_objects::images;
//primero para la visualización de los productos tenemos que obtener el producto
pub async fn get_product(user_id:&Uuid,product_id:&Uuid, pool:&DbPool) -> Result<ProductEdit2,Error>{
    let product = get_product_seller(&user_id,&product_id, &pool).await?;
    let product2 = get_product_with_images1(&user_id,&product_id, &pool).await?;
    let product_variations_start = get_product_variations(&product_id,&pool).await?;
    
    //obtener las variaciones correctamente
    let product_variations:Vec<Variation> = product_variations_start
        .iter()
        .map(|var| {
            //tengo que obtener el json
            let vars:Vec<Attribute> = match &var.attributes{
                Some(vars_xtra) => serde_json::from_value(vars_xtra.clone()).unwrap(),
                None=>Vec::new()
            };
            Variation{
            id: var.id.clone(),
            attributes: vars,
            status: var.status
        }})
        .collect();

    let variations_x = Variations{
        variations:product_variations
    };

    let general = General{
        name: product2.name.unwrap_or("".to_string()),
        description: product2.description.unwrap_or("".to_string()),
        brand: product2.brand.unwrap_or("".to_string()),
        status: product2.status.unwrap_or(0),
    };
    //si es null que devuelva un vector vacio

    let images = match product2.images_product{
        Some(imgs) =>{
            let final_images:Vec<ImageCategories2> = serde_json::from_value(imgs).unwrap();
            final_images    
        },
        None =>  Vec::new()
    };
    
    /*Images{
        vec: match product.images{
            Some(imgs) => {
                let images = 
            },
            None => Vec::new()
        }
    };*/

    let product_edit = ProductEdit2{
        id: product_id.clone(),
        general,
        images,
        variations: variations_x
    };

    Ok(product_edit)
}

//creo un struct por cada apartado del producto?
