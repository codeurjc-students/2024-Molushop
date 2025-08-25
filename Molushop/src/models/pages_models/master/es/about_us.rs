use rinja::Template;
//use crate::models::components::product_card_model::ProductCardData;

#[derive(Template,Clone,Debug)]
#[template(path="pages/master/es/about_us/about_us.html")]
pub struct AboutUs{
    pub page_name:String
}
