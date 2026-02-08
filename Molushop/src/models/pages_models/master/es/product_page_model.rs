use rinja::Template;
use crate::models::components::product_card_model::ProductCardData;
use crate::models::components::nav1_model::*;
use crate::models::components::login_base_model::LoginProductData;
use crate::models::components::product_detail_principal_model::ProductDetailPrincipalData;


#[derive(Template,Clone,Debug)]
#[template(path="pages/master/es/product/product_page.html")]
pub struct ProductPage{
    pub user_logged: bool,
    pub page_name:String,
    pub nav1:Nav1Data,
    pub login_base_data:LoginProductData,
    pub product_data: ProductDetailPrincipalData
}