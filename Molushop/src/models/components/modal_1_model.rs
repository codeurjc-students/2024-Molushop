use rinja::Template;
use serde::Serialize;
use crate::models::data_transfer_objects::product::{Product,RoutesProductPanelGroup};

#[derive(Template,Clone,Debug)]
#[template(path = "components/modal-1/templates/modal-1.html")]
pub struct Modal1{
    pub text: String,
}

#[derive(Serialize,Clone,Debug)]
pub struct Routes{
    pub base: &'static str,
}
