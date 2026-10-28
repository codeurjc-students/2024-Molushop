use askama::Template;

#[derive(Template,Clone,Debug)]
#[template(path = "components/modal/templates/confirm.html")]
pub struct ConfirmModal{
    pub method: String,
    pub message: String,
    pub endpoint: String,
    pub target: String,
    pub swap: String,
    pub hyperscript_action:String,
    pub htmx_active: bool
}
