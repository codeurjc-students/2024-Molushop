use askama::Template;

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use bigdecimal::BigDecimal;

#[derive(Template,Clone,Debug)]
#[template(path = "components/edit_product_variation/templates/edit_product_variation.html")]
pub struct EditProductVariation{
    pub variation: ProductVariationEdit,
    pub routes_edit_product: &'static Routes,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Routes{
    pub edit_general: &'static str,
    pub edit_price: &'static str,
    pub edit_stock: &'static str,
    //pub edit_images: &'static str,
    pub new_identifier: &'static str,
    pub edit_identifiers: &'static str,
    pub warning: &'static str,
    pub delete_identifier: &'static str
    
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductVariationEdit{
    pub id: Uuid,
    pub general: General,
    pub prices: Prices,
    pub stock: Stock,
    pub identifiers: Identifiers,
    //pub images: Images,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stock{
    pub number:i32
    //images
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct General{
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub status: i16,
    //images
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attribute{
    pub name:String,
    pub value:String
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Prices{
    pub actual_price: BigDecimal, // o bigdecimal ?
    pub sale_active: bool,
    pub sale_price: BigDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Identifiers{
    pub identifiers: Vec<Identifier>,
    pub name_options:Vec<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Identifier{
    pub name: String,
    pub value: String,
}


#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct PriceForm{
    pub regular: String,
    pub discount_active: String,
    pub discount: String,
    pub option: String,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct PriceData{
    pub regular: BigDecimal,
    pub discount_active: bool,
    pub discount: BigDecimal,
    pub option: i16,
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct StockForm{
    pub stock: String
}
//hacer un impl a PriceData
/* 
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Images{
    pub images: Vec<ImageCategories>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageCategories{
    pub url: String,
    pub tipo: String,
}*/
