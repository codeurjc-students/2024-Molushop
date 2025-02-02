use rinja::Template;
use crate::models::data_transfer_objects::product::{Product,RoutesProductPanelGroup};

use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Template,Clone,Debug)]
#[template(path = "components/edit_product/templates/edit_product.html")]
pub struct EditProduct{
    pub product: ProductEdit,
    pub routes_edit_product: &'static Routes,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Routes{
    pub edit_general: &'static str,
    pub edit_images: &'static str,
    //pub delete_product_modal: &'static str,
    //images
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductEdit{
    pub id: Uuid,
    pub general: General,
    pub images: Images,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct General{
    pub name: String,
    pub description: String,
    pub brand: String,
    pub status: i16,
    //images
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


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Categories{

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Prices{
    pub prices: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stock{
    pub stock: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Advanced{
    pub purchase_note: String,

}


