use askama::Template;
use crate::models::components::nav1_model::*;
use crate::models::components::login_base_model::LoginProductData;

#[derive(Template,Clone,Debug)]
#[template(path="pages/about_us/about_us.html")]
pub struct AboutUsPage{
    pub user_logged: bool,
    pub page_name:String,
    pub nav1:Nav1Data,
    pub login_base_data:LoginProductData
}
