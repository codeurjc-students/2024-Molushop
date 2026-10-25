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
use askama::Template;

/// `Ok(None)` si el producto no está a la venta: no existe, no está activo o
/// su tienda no está activa (la consulta no devuelve fila). La ruta responde
/// entonces 404 con la página de "no disponible".
pub async fn get_product_object(product_id:&Uuid,pool:&DbPool) -> Result<Option<ProductDetailPrincipalData>, diesel::result::Error>{
    match get_product_for_page(&product_id,&pool).await {
        Ok(product) => Ok(Some(ProductDetailPrincipalData::from(product))),
        Err(diesel::result::Error::NotFound) => Ok(None),
        Err(e) => {
            println!("Error fetching product: {:?}", e);
            Err(e)
        }
    }
}