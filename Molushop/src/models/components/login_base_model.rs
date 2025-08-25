use rinja::Template;

use serde::{Serialize,Deserialize};
use uuid::Uuid;
use bigdecimal::BigDecimal;

#[derive(Template,Clone,Debug)]
#[template(path="components/login_base/templates/login_base.html")]
pub struct LoginProduct{
    pub login_base_data: LoginProductData
}

#[derive(Debug, Clone)]
pub struct LoginProductData{
    pub routes: &'static Routes
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Routes{
    pub create_user: &'static str,
    pub login: &'static str
    
}

///json
/// 
#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct FormRegister{
    pub username: String,
    pub email: String,
    pub pass: String,
    pub pass2: String
}

#[derive(Serialize,Deserialize,Debug,Clone)]
pub struct FormLogin{
    pub username: String,
    pub pass: String,
}