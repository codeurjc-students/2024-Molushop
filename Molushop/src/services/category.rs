use super::servicesX::establish_connection;
use diesel::result::Error;
use uuid::Uuid;
//use crate::models::models_x::{Category,NewBaseUser,NewProduct,Products,ProductForm};
use crate::models::models_x::Category;


use diesel::prelude::*;

pub fn get_name(id_category:&Uuid) -> Result<String, Error>{
    use crate::schema::products::dsl::*;
    let connection = &mut establish_connection();
    let result = products.filter(id.eq(id_category)).select(name).first::<String>(connection);
    //let result = products.filter(id.eq(id_product)).first::<Products>(connection);
    result
}

//obtener categoria, luego obtener los datos que queremos
pub fn get_category(id_category:&String) -> Result<Category,Error> {
    use crate::schema::category::dsl::*;
    let connection = &mut establish_connection();
    let result = category.filter(id.eq(id_category)).first::<Category>(connection);
    result
}