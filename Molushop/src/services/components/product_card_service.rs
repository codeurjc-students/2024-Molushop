
use crate::models::components::product_card_model::*;
use crate::services::product_service::get_product;
use crate::services::price_service::get_product_variation_prices;
use crate::constants::urls::PRODUCT_URL_PREFIX;
use crate::constants::money::DEFAULT_CURRENCY;
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
            tracing::error!(product_id = %id, error = ?e, "could not load the product card");
            return ProductCardData::default(); // O maneja el error de otra manera
        }
    };
    //obtener la imagen principal?
    //obtener el precio más bajo? 
    //poner un precio base y que luego aplique a todas las variaciones?
    //Poner siempre el precio más bajo de las variaciones del producto?
    
    //obtener el precio
    let prices = match get_product_variation_prices(&id,&pool).await{
        Ok(prices)=>prices,
        Err(e)=>{
            tracing::error!(product_id = %id, error = ?e, "could not load the product card prices");
            return ProductCardData::default();
        }
    };
    let price = if !prices.is_empty(){
        prices[0].price.clone()
    }else{
        //crear un data BigDecimal nuevo
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
        id: product.id,
        name:product.name,
        image:img_url,
        price:Some(price),
        currency: DEFAULT_CURRENCY.to_string(),
        url: format!("{}{}",PRODUCT_URL_PREFIX,product.id),
        // Este camino (tarjetas sueltas por id) no consulta las opiniones: la
        // tarjeta sale sin estrellas. Las del listado vienen de get_product_cards.
        rating_average: 0.0,
        rating_count: 0,
        is_favorite: false,
        available: true,
    }
}
pub async fn get_product_card_render(id:&Uuid,pool:&DbPool)->String{
    //llamda a la base de datos
    //let product = get_product(id,pool).await;
    let data = get_product_card_object(&id,&pool).await;

    let render  = ProductCard{
        product:data,
        i18n: crate::i18n::I18n::default_ref(),
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