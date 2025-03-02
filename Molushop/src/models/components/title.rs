use rinja::Template;
use serde::Serialize;
use crate::models::data_transfer_objects::product::{Product,RoutesProductPanelGroup};

#[derive(Template,Clone,Debug)]
#[template(path = "components/title/templates/title.html")]
pub struct Title{
    pub title: String,
}
