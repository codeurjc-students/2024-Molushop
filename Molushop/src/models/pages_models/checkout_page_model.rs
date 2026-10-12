use askama::Template;
use crate::models::components::nav1_model::*;
use crate::models::components::login_base_model::LoginProductData;
use crate::models::components::checkout_v1::CheckoutData;

/// Página de checkout.
#[derive(Template,Clone,Debug)]
#[template(path="pages/checkout/checkout.html")]
pub struct CheckoutPage{
    pub user_logged: bool,
    pub page_name:String,
    pub nav1:Nav1Data,
    pub login_base_data:LoginProductData,
    pub checkout:CheckoutData
}
