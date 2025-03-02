use rinja::Template;

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use bigdecimal::BigDecimal;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

#[derive(Template,Clone,Debug)]
#[template(path = "components/edit_product_variation/templates/identifier.html")]
pub struct IdentifierBase{
    pub variation_id: String,
    pub identifier: Identifier, 
    pub number: i32,
    pub routes:Routes,
    pub name_options: Vec<String>
}

use crate::services::servicesX::get_identifier_options_var;
use crate::controllers::components::edit_product_variation_controller::ROUTES;
impl IdentifierBase{
    pub fn empty()->Self{
        Self { 
            variation_id: "variation".to_string(),
            identifier: Identifier{
                name:String::new(),
                value: String::new()
            },
            number:0,
            routes: Routes{
                warning:"",
                delete_identifier:""

            },
            name_options: vec!["SKU".to_string(), "EAN".to_string(), "UPC".to_string(), 
            "ISBN".to_string(), "MPN".to_string(), "GTIN".to_string()]
        }
    }
    pub async fn base(pool:&DbPool)->Self{
        Self { 
            variation_id: "variation".to_string(),
            identifier: Identifier{
                name:String::new(),
                value: String::new()
            },
            number:0,
            routes: Routes{
                warning:&ROUTES.warning,
                delete_identifier:&ROUTES.delete_identifier

            },
            name_options: get_identifier_options_var(pool).await.unwrap_or_else(|_| vec![
                "SKU".to_string(), "EAN".to_string(), "UPC".to_string()])
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Identifier{
    pub name:String,
    pub value:String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Routes{
    pub warning: &'static str,
    pub delete_identifier: &'static str
    
}