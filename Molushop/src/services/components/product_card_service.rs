
use crate::models::components::product_card_model::*;
use crate::services::servicesX::*;
use crate::constants::urls::PRODUCT_URL_PREFIX;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use uuid::Uuid;
type DbPool = Pool<AsyncPgConnection>;
use askama::Template;
use bigdecimal::BigDecimal;
use serde::{Serialize, Deserialize};
//le pasamos los parámetros que necesita?
pub async fn get_product_card_object(id:&Uuid,pool:&DbPool)->ProductCardData{
    //llamda a la base de datos
    //let product = get_product(id,pool).await;
    
    let product = match get_product(&id, &pool).await {
        Ok(p) => p,
        Err(e) => {
            println!("Error fetching product: {:?}", e);
            return ProductCardData::default(); // O maneja el error de otra manera
        }
    };
    //obtener la imagen principal?
    //obtener el precio más bajo? 
    //poner un precio base y que luego aplique a todas las variaciones?
    //Poner siempre el precio más bajo de las variaciones del producto?
    
    //obtener el precio
    let precios = match get_variations_prices_product(&id,&pool).await{
        Ok(prices)=>prices,
        Err(e)=>{
            println!("Error obteniendo los precios {:?}", e);
            return ProductCardData::default();
        }
    };
    let precio = if !precios.is_empty(){
        precios[0].price.clone()
    }else{
        //crear un objeto BigDecimal nuevo
        BigDecimal::from(0)
    };
    //obtener las imagenes
    let images = match product.images{
        Some(imgs) =>{
            let final_images:Images = serde_json::from_value(imgs).unwrap();
            final_images    
        },
        None => Images{images: Vec::new()}
    };
    let img_url = if !images.images.is_empty(){
        images.images[0].url.clone()
    }else{
        //url generica
        "url".to_string()
    };

    ProductCardData{
        name:product.name,
        image:img_url,
        price:precio,
        url: format!("{}{}",PRODUCT_URL_PREFIX,product.id)
    }
}
pub async fn get_product_card_render(id:&Uuid,pool:&DbPool)->String{
    //llamda a la base de datos
    //let product = get_product(id,pool).await;
    let objeto = get_product_card_object(&id,&pool).await;

    let render  = ProductCard{
        product:objeto
    }.render().unwrap();
    render
    
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Images{
    pub images: Vec<ImageCategories>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageCategories{
    pub url: String,
    pub tipo: String,
}