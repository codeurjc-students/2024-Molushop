use askama::Template;
use crate::i18n::I18n;
use crate::models::components::product_card_model::ProductCardData;
use crate::models::components::nav1_model::*;
use crate::models::components::login_base_model::LoginProductData;
use crate::models::components::product_detail_principal_model::ProductDetailPrincipalData;
use crate::models::components::product_reviews_v1::ProductReviewsData;


#[derive(Template,Clone,Debug)]
#[template(path="pages/product/product.html")]
pub struct ProductPage{
    pub user_logged: bool,
    pub page_name:String,
    pub nav1:Nav1Data,
    pub login_base_data: LoginProductData,
    /// El idioma de la petición (src/i18n.rs): `<html lang>`, enlaces y textos.
    pub i18n: I18n,
    pub product_data: ProductDetailPrincipalData,
    pub reviews: ProductReviewsData
}