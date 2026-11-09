use askama::Template;
use crate::i18n::I18n;
use crate::models::components::nav1_model::*;
use crate::models::components::login_base_model::LoginProductData;
use crate::models::components::cart_total_v1::CartTotalData;

#[derive(Template,Clone,Debug)]
#[template(path="pages/cart/cart.html")]
pub struct CartPage{
    pub user_logged: bool,
    pub page_name:String,
    pub nav1:Nav1Data,
    pub login_base_data: LoginProductData,
    /// El idioma de la petición (src/i18n.rs): `<html lang>`, enlaces y textos.
    pub i18n: I18n,
    pub cart_total:CartTotalData
}
