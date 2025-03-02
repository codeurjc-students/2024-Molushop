use uuid::Uuid;
use diesel::result::Error;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use crate::controllers::createProduct::product;
use crate::models::components::edit_product_variation_model::{ProductVariationEdit,General,Attribute,Prices,Identifier,Identifiers};
use crate::services::servicesX::{get_identifier_options_var,get_product_variation,get_variation_price,get_variation_identifiers};
use bigdecimal::BigDecimal;

//hacer un vec de variations
 
pub async fn start(user_id:&Uuid,product_var_id:&Uuid, pool:&DbPool) -> Result<ProductVariationEdit,Error>{
    //añadir verificación que el producto tenga asociado el usuario!!!

    //primero obtener la variación //se va a obtener a partir del id de la variación directamente
    //va a ser editable por el usuario --> que el usuario
    let product_var = get_product_variation(&product_var_id, pool).await?;
    //obtener los atributos del usuari    

    //obtener el vector  de Atributos:
    
    let attributes:Vec<Attribute> = match product_var.attributes{
        Some(att) =>{
            let aaa:Vec<Attribute> =serde_json::from_value(att).unwrap();
            aaa
        },
        None => vec![]
    };
    //obtener el nomrbe a partir de los valores del 
    let name = concatenate_attribute_values(&attributes);

    let general = General{
        //el nombre tiene que ser la combianción de atributos
        name,
        attributes,
        status : product_var.status
    };
    //obtener el precio actual de la variación a partir de el id de la variación y de su moneda, en este caso EUR
    let currency = String::from("EUR");

    let price_base = get_variation_price(&product_var_id,&currency,&pool).await?;

    let big_decimal_aux = BigDecimal::from(20);

    let prices = Prices{
        actual_price: price_base.price,
        sale_price: big_decimal_aux
    };

    let identifiers_base = get_variation_identifiers(&product_var_id,&pool).await?;
    //Ahora con esos identifiers --> hacer 
    let identifiers = identifiers_base
        .iter()
        .map(|a| Identifier{name:a.identifier.clone(),value:a.value.clone()})
        .collect();

    let name_options = get_identifier_options_var(&pool).await?;

    let identifiers = Identifiers{
        identifiers,
        name_options
    };

    /* E
    let images = Images{
        images:
    }*/

    let product_variation_edit = ProductVariationEdit{
        id: product_var_id.clone(),
        general,
        prices,
        identifiers
    };

    Ok(product_variation_edit)
}
    
pub fn concatenate_attribute_values(attributes: &[Attribute]) -> String {
    attributes
        .iter()
        .map(|attr| attr.value.as_str())
        .collect::<Vec<&str>>()
        .join(" ")
}