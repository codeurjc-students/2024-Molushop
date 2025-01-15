// logica de implementación
// obtener productos y variaciones
use diesel::result::Error;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use crate::models::data_transfer_objects::product::Product;
type DbPool = Pool<AsyncPgConnection>;


pub async fn get_products(pool: &DbPool) -> Result<Vec<Product>,Error>{
    //obtener los productos y sus respectivas variaciones 
    Ok(vec![])

}

// obtener los productos de un vendedor 