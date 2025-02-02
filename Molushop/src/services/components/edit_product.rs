use serde::{Serialize, Deserialize};
use diesel::result::Error;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use crate::models::data_transfer_objects::product::Product;
use crate::models::models_x::{ProductVariation,Category};
use crate::models::data_transfer_objects::product;
//use crate::schema::products::variations;
use crate::services::servicesX::{get_product_seller};
type DbPool = Pool<AsyncPgConnection>;
use uuid::Uuid;
use crate::models::components::edit_product::{ProductEdit,General,Images,Prices};
use crate::models::data_transfer_objects::images;
//primero para la visualización de los productos tenemos que obtener el producto
pub async fn get_product(user_id:&Uuid,product_id:&Uuid, pool:&DbPool) -> Result<ProductEdit,Error>{
    let product = get_product_seller(user_id,product_id, pool).await?;
    //obtener las imagenes del producto

    let general = General{
        name: product.name,
        description: product.description,
        brand: product.brand,
        status: product.status,
    };
    //si es null que devuelva un vector vacio

    let images = match product.images{
        Some(imgs) =>{
            let final_images:Images = serde_json::from_value(imgs).unwrap();
            final_images
        },
        None => Images{images: Vec::new()}
    };
    
    /*Images{
        vec: match product.images{
            Some(imgs) => {
                let images = 
            },
            None => Vec::new()
        }
    };*/

    let product_edit = ProductEdit{
        id: product_id.clone(),
        general,
        images,
    };

    Ok(product_edit)
}

//creo un struct por cada apartado del producto?