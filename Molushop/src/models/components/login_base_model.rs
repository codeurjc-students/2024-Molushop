use rinja::Template;

use serde::{Serialize,Deserialize};
use uuid::Uuid;
use bigdecimal::BigDecimal;

#[derive(Template,Clone,Debug)]
#[template(path="components/login_base/templates/login_base.html")]
pub struct LoginProduct{
    
}