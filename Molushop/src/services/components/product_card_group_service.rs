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
use rinja::Template;
use bigdecimal::BigDecimal;
use serde::{Serialize, Deserialize};
use futures_util::future::join_all;

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

pub async fn get_product_card_group_render(ids:Vec<&Uuid>,pool:&DbPool)->String{
    
    ProductCardGroup{
        products:get_product_card_group_object_concurrent(ids,pool).await
    }.render().unwrap()
}