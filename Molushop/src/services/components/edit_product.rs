use diesel::result::Error;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use crate::services::variation_service::get_product_variations;
use crate::services::product_service::get_product_with_images;
type DbPool = Pool<AsyncPgConnection>;
use uuid::Uuid;
use crate::models::components::edit_product::{ProductEdit2,ImageCategories2,General,Variations,Variation,Attribute};
//primero para la visualización de los productos tenemos que obtener el producto
/// No comprueba propiedad: el handler llama antes a `can_edit_product`.
pub async fn get_product(product_id:&Uuid, pool:&DbPool) -> Result<ProductEdit2,Error>{
    let product2 = get_product_with_images(&product_id, &pool).await?;
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

    let specs = crate::services::product_specs_service::get_specs(product_id, pool).await?;

    let product_edit = ProductEdit2{
        id: product_id.clone(),
        general,
        images,
        variations: variations_x,
        specs,
    };

    Ok(product_edit)
}

//creo un struct por cada apartado del producto?
