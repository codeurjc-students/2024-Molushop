use actix_web::{cookie::Cookie, get, post, web, HttpRequest, HttpResponse, Scope};
use lazy_static::lazy_static;
use askama::Template;
use crate::models::components::login_base_model::*;
use crate::services::components::login_base_service::*;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use super::scope::SCOPE_COMPONENTS;
use crate::services::user_service::{insert_user, get_user_by_username};
use crate::services::session_service::revoke_session;
use crate::middleware::auth::{Auth, SessionData};
use crate::constants::urls::HOME_URL;
use crate::i18n::I18n;
use actix_web::HttpMessage;

pub static SCOPE1: &str = "/login-base-login";
pub static SCOPE2: &str = "/login-base-register";

lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza

    pub static ref ROUTES: Routes = Routes{
        //delete_product: Box::leak(format!("{}{}/delete-product",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
        //delete_product_modal: Box::leak(format!("{}{}/delete-product-modal",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
        create_user: Box::leak(format!("{}{}/create-user",SCOPE_COMPONENTS, SCOPE2).into_boxed_str()),
        login: Box::leak(format!("{}{}/login",SCOPE_COMPONENTS, SCOPE1).into_boxed_str()),
        logout: Box::leak(format!("{}{}/logout",SCOPE_COMPONENTS, SCOPE1).into_boxed_str())
    };

}

//
use crate::middleware::prueba::*;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};

//en el scope se puede poner el middleware
pub fn scope1() -> Scope<impl ServiceFactory<ServiceRequest, Config = (), Response = ServiceResponse, Error = actix_web::Error, InitError = ()>> {
    web::scope(SCOPE1)
    .wrap(SayHi::new())
    .configure(config1)
}

fn config1(cfg: &mut web::ServiceConfig) {
    //cfg.service(products_panel_prueba);
    cfg
        .service(login)
        .service(spawn)
        // Auth solo en el logout: el login, por definición, llega sin sesión.
        .service(
            web::resource("/logout")
                .wrap(Auth::new())
                .route(web::post().to(logout))
        );
}
pub fn scope2() -> Scope {
    web::scope(SCOPE2)
        .configure(config2)
}
fn config2(cfg: &mut web::ServiceConfig) {
    //cfg.service(products_panel_prueba);
    cfg
        .service(create_user);
}

/// Cierra la sesión del navegador que lo pide: revoca su fila en `user_sessions` y borra la
/// cookie. Sin sesión válida (sin cookie, caducada o ya revocada) no hay nada que revocar y
/// solo se borra la cookie: para el usuario el resultado es el mismo.
///
/// Si falla la revocación NO se borra la cookie: el token seguiría siendo válido en BD, así
/// que decir "sesión cerrada" sería mentira. Se avisa y el usuario puede reintentar.
async fn logout(i18n: I18n, req: HttpRequest, pool_data: web::Data<DbPool>) -> HttpResponse {
    let session = req.extensions().get::<SessionData>().cloned();

    if let Some(session) = session {
        if let Err(e) = revoke_session(&session.jti, &session.id, pool_data.get_ref()).await {
            println!("Error al revocar la sesión: {}", e);
            let modal_render = MessageModal::translated(&i18n, "auth-logout-failed").render().unwrap();
            return HttpResponse::InternalServerError().body(modal_render)
        }
    }

    let mut cookie = Cookie::new("sesion_token", "");
    cookie.set_path("/");
    cookie.make_removal();

    // HX-Redirect y no recargar: en /orders o /checkout la página sin sesión quedaría a medias.
    // A la portada del idioma de la página.
    HttpResponse::Ok()
        .cookie(cookie)
        .insert_header(("HX-Redirect", i18n.url(HOME_URL)))
        .finish()
}


#[get("/spawn")]
async fn spawn()->HttpResponse	{
    //service que me cargue datos del login
    //parámetros que desean --> 
    let render_login = get_login_base_rendered();
    HttpResponse::Ok().body(render_login)

}

use crate::models::components::message_modal::MessageModal;
use crate::services::auth::{hash_password,verify_password};

#[post("/create-user", wrap = "actix_web::middleware::from_fn(crate::middleware::rate_limit::limit_register)")]
async fn create_user(i18n: I18n, form:web::Json<FormRegister>,pool_data:web::Data<DbPool>)->HttpResponse{
    //reciba un json -->
    let form_register =form.into_inner();
    let pool = pool_data.get_ref();
    //hacer verificaciones
    // 1. Estructura de correo correcta?
    // Validar los datos
    let is_validated = form_register_validation(&form_register,&pool).await.unwrap();
    //Si esta todo correcto, crear el usuario


    if !is_validated.is_valid {
        let modal_render = MessageModal::translated(&i18n, "auth-register-invalid").render().unwrap();
        return HttpResponse::BadRequest().body(modal_render) 
    };

    //Aqui llamar al servicio de hasheado de contraseña
    let hashed_pass = match hash_password(&form_register.pass){
        Ok(pass)=> pass,
        Err(e)=>{
            println!("Error! -> {}",e);
            let modal_render = MessageModal::translated(&i18n, "auth-error").render().unwrap();
            return HttpResponse::InternalServerError().body(modal_render)
        }
    };

    let result_new_user = insert_user(&form_register.username,&form_register.email,&hashed_pass, pool).await;

    match result_new_user{
        Ok(_)=>{
            let modal_render = MessageModal::translated(&i18n, "auth-register-ok").render().unwrap();
            return HttpResponse::Ok().body(modal_render)
        },
        Err(e)=>{
            println!("Error! -> {}",e);
            let modal_render = MessageModal::translated(&i18n, "auth-register-failed").render().unwrap();
            return HttpResponse::InternalServerError().body(modal_render)
        }
    }
}

use crate::services::session_service::start_session;

#[post("/login", wrap = "actix_web::middleware::from_fn(crate::middleware::rate_limit::limit_login)")]
async fn login(i18n: I18n, form:web::Json<FormLogin>,pool_data:web::Data<DbPool>,req:HttpRequest)->HttpResponse{
    //verificar que el ussuario y contraseña existen
    let form_login = form.into_inner();
    let pool = pool_data.get_ref();
    let user_data= match get_user_by_username(&form_login.username,&pool).await{
        Ok(user)=> user,
        Err(e)=>{
            println!("Error! -> {}",e);
            // El mismo aviso que con la contraseña mala: no se dice qué usuarios existen.
            let modal_render = MessageModal::translated(&i18n, "auth-login-wrong").render().unwrap();
            return HttpResponse::Unauthorized().body(modal_render)
        }
    };
    let hashed_pass = user_data.password;
    match verify_password(&form_login.pass,&hashed_pass){
        Ok(res)=>{
            if res{
                // Contraseña correcta: token, fila en `user_sessions` y cookie.
                match start_session(&user_data.id, &req, pool).await {
                    Ok(cookie) => {
                        let modal_render = MessageModal::translated(&i18n, "auth-login-ok").render().unwrap();
                        return HttpResponse::Ok()
                            .cookie(cookie)
                            .body(modal_render)
                    },
                    Err(e) => {
                        println!("Login: {}", e);
                        let modal_render = MessageModal::translated(&i18n, "auth-login-failed").render().unwrap();
                        return HttpResponse::InternalServerError().body(modal_render)
                    }
                }
            }else {
                let modal_render = MessageModal::translated(&i18n, "auth-login-wrong").render().unwrap();
                return HttpResponse::Unauthorized().body(modal_render)    
            }
        },
        Err(e)=>{
            println!("Error! -> {}",e);
            let modal_render = MessageModal::translated(&i18n, "auth-error").render().unwrap();
            return HttpResponse::InternalServerError().body(modal_render)
        }
    }

}

