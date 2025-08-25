use crate::models::components::login_base_model::*;
use crate::controllers::components::login_base_controller::ROUTES;
use crate::schema::base_user::password;
use rinja::Template;
use crate::utils::validations::*;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
use diesel::result::Error;
type DbPool = Pool<AsyncPgConnection>;

pub fn get_login_base_rendered()->String{
    //esto devuelve un render completo del componente
    let login_product= LoginProduct{
        login_base_data: LoginProductData{
            routes: &ROUTES
        }
    };
    login_product.render().unwrap()
    
}

pub fn get_login_base_model_data()->LoginProductData{
    //aqui se cargaría lo que viene siendo el Modelo que se cargará dentro del modelo template
    LoginProductData{
        routes: &ROUTES
    }
    
}
pub struct ValidationResult{
    pub is_valid: bool,
    pub email_valid: bool,
    pub password_match: bool,
    pub password_strong: bool,
    pub email_unique: bool,
    pub username_unique: bool
}

use crate::services::servicesX::{check_mail,check_username};
pub async fn form_register_validation(form:&FormRegister,pool:&DbPool)-> Result<ValidationResult, Error>{
    let pass1= &form.pass;
    let pass2= &form.pass2;
    let email = &form.email;
    let username  = &form.username;
    //todas estas cosas también se tienen que validar en el front
    //validar los datos
    //1. Validacion estructura de correo electrónico
    let email_valid= is_valid_email(&email);
    //2. Verificar que las contraseñas son iguales
    let password_match = pass1.eq(&pass2.clone());
    //3. Validación de la fortaleza de la contraseña
    let password_strong = is_password_strong(&pass1);
    //4. Validacion de la unicidad del email
    let email_unique = !check_mail(&email, &pool).await?;
    let username_unique = !check_username(username, pool).await?;
    //5. Validación del nombre de usuario
    // - Que no esté en la base de datos?

    Ok(
        ValidationResult { 
            is_valid: email_valid&&password_match&&password_strong&&email_unique&&username_unique, 
            email_valid,
            password_match, 
            password_strong,
            email_unique,
            username_unique
        }
    )
    
}

//función para meeter los datos necesarios para la función de la base de datos
//la cuestión es: más de un componente puede hacer el login?