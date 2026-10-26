use crate::models::components::{
    product_card_model::*,
    product_card_group_model::*
};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use uuid::Uuid;
type DbPool = Pool<AsyncPgConnection>;
use crate::services::{
    servicesX::*,
    components::product_card_service::*
};
use askama::Template;
use bigdecimal::BigDecimal;
use serde::{Serialize, Deserialize};
use futures_util::future::join_all;
use crate::models::models_x::{ProductCard1};
use crate::controllers::components::product_card_group_controller::ROUTES as PRODUCT_CARD_GROUP_ROUTES;

//esta especie de contructor tendrá una lista de uuid que se quieran mostrar 
pub async fn get_product_card_group_object(ids:Vec<&Uuid>,pool:&DbPool)->Vec<ProductCardData>{
    //Crear un vector vacío
    let mut pc_vector:Vec<ProductCardData> = Vec::new();
    for id in ids {
        pc_vector.push(get_product_card_object(id,pool).await)
    }
    pc_vector
}

pub async fn get_product_card_group_object_concurrent(
    ids: Vec<&Uuid>,
    pool: &DbPool
) -> Vec<ProductCardData> {
    // Crea un vector de futures, donde cada future es una llamada a get_product_card_object
    let futures: Vec<_> = ids
        .into_iter() // Mueve la propiedad del vector para evitar clonar los Uuid
        .map(|id| get_product_card_object(id, pool))
        .collect();

    // Espera a que todos los futures se completen de forma concurrente
    join_all(futures).await
}
//luego se podrá 
/// Productos por tanda en la portada.
pub const HOME_PAGE_SIZE: i64 = 24;

/// Una tanda de la portada o de la búsqueda (`page` desde 1) y, si quedan
/// más, la URL de la siguiente, con el mismo filtro. Se pide una tarjeta de
/// más para saber si hay siguiente.
pub async fn get_product_card_group_page(page: i64, filter: &CardFilter, pool:&DbPool) -> (Vec<ProductCardData>, Option<String>){
    let page = page.max(1);
    let patterns = filter.like_patterns();
    match get_product_cards1(HOME_PAGE_SIZE + 1, (page - 1) * HOME_PAGE_SIZE, &patterns, filter.category.as_deref(), pool).await{
        Ok(mut data) => {
            let more = data.len() as i64 > HOME_PAGE_SIZE;
            data.truncate(HOME_PAGE_SIZE as usize);
            let cards = data.into_iter().map(ProductCardData::from).collect();
            let more_url = more.then(|| filter.url(&PRODUCT_CARD_GROUP_ROUTES.page, Some(page + 1)));
            (cards, more_url)
        },
        Err(e) => {
            println!("Error getting the product cards!! --> {}",e);
            (Vec::new(), None)
        }
    }
}

pub async fn get_product_card_group_render(ids:Vec<&Uuid>,pool:&DbPool)->String{
    
    ProductCardGroup{
        products:get_product_card_group_object_concurrent(ids,pool).await,
        more_url: None,
        i18n: crate::i18n::I18n::default_ref(),
    }.render().unwrap()
}

pub async fn get_product_card_group_render_all(i18n: &crate::i18n::I18n, pool:&DbPool) -> String{
    //Siempre para los result hacer un match, nunca usar unwrap() --> mala practica
    let (products, more_url) = get_product_card_group_page(1, &CardFilter::default(), pool).await;
    let object_product_card = ProductCardGroup{ products, more_url, i18n };
    match object_product_card.render(){
        Ok(render) => render,
        Err(e) => {
            println!("Error en el renderizado de ProductCardGroup! --> {}",e);
            return "".to_string()
        }
    }
}