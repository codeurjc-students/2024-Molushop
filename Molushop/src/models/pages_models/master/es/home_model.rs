//vam
use rinja::Template;
use crate::models::components::product_card_model::ProductCardData;
use crate::models::components::nav1_model::*;
use crate::models::components::login_base_model::LoginProductData;

#[derive(Template,Clone,Debug)]
#[template(path="pages/master/es/home/home_es.html")]
pub struct Home{
    pub user_logged: bool,
    pub page_name:String,
    pub product:ProductCardData,
    pub pcard1:String,
    pub nav1:Nav1Data,
    pub login_base_data:LoginProductData
}


//aqui los elementos de la pagina? tipo variables de componentes?
