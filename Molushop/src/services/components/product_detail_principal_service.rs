use crate::models::{
    components::product_detail_principal_model::*,
    models_x::ProductForPage1
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

pub async fn get_product_object(product_id:&Uuid,pool:&DbPool) -> ProductDetailPrincipalData{
    //Primero obtener el objeto de la base de datos
    let product = match get_product_for_page(&product_id,&pool).await {
        Ok(p) => p,
        Err(e) => {
            println!("Error fetching product: {:?}", e);
            return ProductDetailPrincipalData::default();
        }
    };
    //una vez obtenido el producto, vamos a ver 

    return ProductDetailPrincipalData::from(product);
}