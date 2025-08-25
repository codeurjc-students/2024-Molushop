use rinja::Template;

use serde::{Serialize,Deserialize};


#[derive(Template,Clone,Debug)]
#[template(path="components/nav1/templates/nav1.html")]
pub struct Nav1{
    //pub login_base_data: Nav1Data
}

#[derive(Debug, Clone)]
pub struct Nav1Data{
    pub routes: &'static Routes
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Routes{
    pub home: &'static str
    
}
