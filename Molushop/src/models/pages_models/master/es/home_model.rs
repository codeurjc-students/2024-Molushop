//vam
use rinja::Template;
use crate::models::components::product_card_model::ProductCardData;

#[derive(Template,Clone,Debug)]
#[template(path="pages/master/es/home/home_es.html")]
pub struct Home{
    pub page_name:String,
    pub product:ProductCardData,
    pub pcard1:String
}


//aqui los elementos de la pagina? tipo variables de componentes?
