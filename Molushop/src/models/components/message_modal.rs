use askama::Template;

#[derive(Template,Clone,Debug)]
#[template(path = "components/modal/templates/message.html")]
pub struct MessageModal{
    pub text: String,
}

impl MessageModal{
    pub fn new(text: &str)-> Self{
        Self{
            text: text.to_string()
        }
    }
    pub fn from_string(text: String)-> Self{
        Self{
            text
        }
    }
}