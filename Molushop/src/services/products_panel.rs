// logica de implementación
// obtener productos y variaciones
use serde::{Serialize, Deserialize};
use diesel::result::Error;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use crate::models::data_transfer_objects::product::Product;
use crate::models::models_x::{ProductVariation,Category,Price};
use crate::models::data_transfer_objects::product;
//use crate::schema::products::variations;
use crate::services::panel_role_service::PanelUser;
use crate::services::servicesX::{
    get_panel_products,
    get_product_categories,
    get_product_variations,
    get_variations_with_prices
};
type DbPool = Pool<AsyncPgConnection>;

/// Productos del listado del panel: el admin ve los de todas las tiendas (con
/// el nombre de la tienda), el vendedor solo los suyos. Sin tienda y sin admin
/// -> lista vacía.
pub async fn get_products(panel_user:&PanelUser,pool: &DbPool) -> Result<Vec<Product>,Error>{

    let seller_filter = if panel_user.is_admin {
        None
    } else {
        let store_ids = panel_user.manageable_store_ids();
        if store_ids.is_empty() {
            return Ok(Vec::new());
        }
        Some(store_ids)
    };
    let products = get_panel_products(seller_filter.as_deref(), pool).await?;
    let mut products_final:Vec<Product> = Vec::new();

    for (product, store_name) in products{
        let categories: Vec<Category> = get_product_categories(&product.id, &pool).await?;
        let categories_names:Vec<String> = categories.iter().map(|c| c.name.clone()).collect();
        let vars: Vec<ProductVariation> = get_product_variations(&product.id, &pool).await?;
        let vars_price: Vec<(ProductVariation,Price)> = get_variations_with_prices(&product.id, &"EUR".to_string(), &pool).await?;
        //let variations:Vec<product::ProductVariation> = Vec::new();
        
        let variation_titles:Vec<String> = match product.variation_titles{
            Some(titles) =>{
                let titles_aux:VariationTitles = serde_json::from_value(titles).unwrap();
                titles_aux.titulos
            },
            None => Vec::new()     
        };
        //let variations: Vec<product::ProductVariation> = vars.iter().map(|v| product::ProductVariation::from(v)).collect();  
        let variations: Vec<product::ProductVariation> = vars_price.iter().map(|(pv,p)| product::ProductVariation::from(pv,p)).collect();  

        let product_final = Product{
            variations: variations,
            name: product.name,
            status: product.status,
            code: product.code,
            variations_titles: variation_titles,
            categories: categories_names,
            id: product.id,
            store_name: if panel_user.is_admin { Some(store_name) } else { None },
        };
        products_final.push(product_final);
    }
        
    Ok(products_final)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VariationTitles{
    pub titulos: Vec<String>,
}


// obtener los productos de un vendedor 