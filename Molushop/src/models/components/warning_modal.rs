use askama::Template;
use serde::Serialize;
use crate::models::data_transfer_objects::product::{Product,RoutesProductPanelGroup};

#[derive(Template,Clone,Debug)]
#[template(path = "components/warning_modal/templates/base.html")]
pub struct WarningModal{
    pub method: String,
    pub message: String,
    pub endpoint: String,
    pub target: String,
    pub swap: String,
    pub hyperscript_action:String,
    pub htmx_active: bool
}
