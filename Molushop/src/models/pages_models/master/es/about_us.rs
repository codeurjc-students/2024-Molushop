use rinja::Template;
//use crate::models::components::product_card_model::ProductCardData;
use crate::models::components::nav1_model::*;

#[derive(Template,Clone,Debug)]
#[template(path="pages/master/es/about_us/about_us.html")]
pub struct AboutUs{
    pub user_logged:bool,
    pub page_name:String,
    pub nav1:Nav1Data
}
