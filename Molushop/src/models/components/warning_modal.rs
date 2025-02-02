use rinja::Template;
use serde::Serialize;
use crate::models::data_transfer_objects::product::{Product,RoutesProductPanelGroup};

#[derive(Template,Clone,Debug)]
#[template(path = "components/warning_modal/templates/base.html")]
pub struct WarningModal{
    pub method: &'static str,
    pub message: &'static str,
    pub endpoint: String,
    pub target: String,
    pub swap: &'static str,
}
