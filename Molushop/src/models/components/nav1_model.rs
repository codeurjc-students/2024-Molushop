use askama::Template;

use serde::{Serialize,Deserialize};


#[derive(Template,Clone,Debug)]
#[template(path="components/nav1/templates/nav1.html")]
pub struct Nav1{
    //pub login_base_data: Nav1Data
    pub nav1: Nav1Data
}

#[derive(Debug, Clone)]
pub struct Nav1Data{
    pub routes: &'static Routes,
    pub name: String,
    pub cart_count: i64,
    /// Si hay sesión. El menú de perfil enseña "Identifícate" o las entradas de la cuenta
    /// según esto, en vez de deducirlo de que `name` venga vacío.
    pub logged: bool
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Routes{
    pub home: String,
    pub cart: String,
    pub orders: String,
    pub logout: String
}
