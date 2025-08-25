use actix_web::{cookie::{time::OffsetDateTime, Cookie}, delete, dev::Transform, get, http::StatusCode, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Scope};
use actix_web::cookie::time::Duration as CookieDuration;
use aws_sdk_s3::primitives::event_stream::Message;
use bigdecimal::BigDecimal;
use diesel::result;
use lazy_static::lazy_static;
use rinja::Template;
use uuid::Uuid;
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::models::models_x::NewUserSession;
use crate::{controllers::createProduct::product, models::components::warning_modal::WarningModal};
use crate::models::components::login_base_model::*;
use crate::services::components::login_base_service::*;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use super::scope::SCOPE_COMPONENTS;
use crate::services::servicesX::{new_user_base,get_user,insert_session_login};
use crate::models::components::login_base_model::*;
use std::collections::HashMap;
use chrono::{Utc, NaiveDateTime, Duration};
use std::time::Duration as DurationTime;

pub static SCOPE1: &str = "/login-base-login";
pub static SCOPE2: &str = "/login-base-register";

lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza
    
    pub static ref ROUTES: Routes = Routes{
        //delete_product: Box::leak(format!("{}{}/delete-product",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
        //delete_product_modal: Box::leak(format!("{}{}/delete-product-modal",SCOPE_COMPONENTS, SCOPE).into_boxed_str()),
        create_user: Box::leak(format!("{}{}/create-user",SCOPE_COMPONENTS, SCOPE2).into_boxed_str()),
        login: Box::leak(format!("{}{}/login",SCOPE_COMPONENTS, SCOPE1).into_boxed_str())
    };

}

//
use crate::middleware::prueba::*;
use actix_web::{
    dev::{ServiceFactory,forward_ready, Service, ServiceRequest, ServiceResponse},
    Error,
};

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
        .service(spawn);
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


#[get("/spawn")]
async fn spawn()->HttpResponse	{
    //service que me cargue datos del login
    //parámetros que desean --> 
    let render_login = get_login_base_rendered();
    HttpResponse::Ok().body(render_login)

}

use crate::models::components::modal_1_model::Modal1;
use crate::services::auth::{hash_password,verify_password};
use crate::models::error::ServiceError;

#[post("/create-user")]
async fn create_user(form:web::Json<FormRegister>,pool_data:web::Data<DbPool>)->HttpResponse{
    //reciba un json -->
    let form_register =form.into_inner();
    let pool = pool_data.get_ref();
    //hacer verificaciones
    // 1. Estructura de correo correcta?
    // Validar los datos
    let is_validated = form_register_validation(&form_register,&pool).await.unwrap();
    //Si esta todo correcto, crear el usuario


    if !is_validated.is_valid {
        let message = "Hay problemas con el formato";
        let modal_render = Modal1::new(message).render().unwrap();
        return HttpResponse::BadRequest().body(modal_render) 
    };

    //Aqui llamar al servicio de hasheado de contraseña
    let hashed_pass = match hash_password(&form_register.pass){
        Ok(pass)=> pass,
        Err(e)=>{
            println!("Error! -> {}",e);
            let message = "Ha habido un problema al hashear la contraseña";
            let modal_render = Modal1::new(message).render().unwrap();
            return HttpResponse::InternalServerError().body(modal_render)
        }
    };

    let result_new_user = new_user_base(&form_register.username,&form_register.email,&hashed_pass, pool).await;

    match result_new_user{
        Ok(num)=>{
            //let message = "Usuario creado correctamente";
            let mensaje = format!("Usuario creado correctamente: {}",num);
            let modal_render = Modal1::new(&mensaje).render().unwrap();
            return HttpResponse::Ok().body(modal_render)
        },
        Err(e)=>{
            println!("Error! -> {}",e);
            let message = "No se ha podido crear el usuario";
            let modal_render = Modal1::new(message).render().unwrap();
            return HttpResponse::InternalServerError().body(modal_render)
        }
    }
}

use crate::services::paseto_token_session_service::*;

#[post("/login")]
async fn login(form:web::Json<FormLogin>,pool_data:web::Data<DbPool>,req:HttpRequest)->HttpResponse{
    //verificar que el ussuario y contraseña existen
    let form_login = form.into_inner();
    let pool = pool_data.get_ref();
    let user_data= match get_user(&form_login.username,&pool).await{
        Ok(user)=> user,
        Err(e)=>{
            println!("Error! -> {}",e);
            let message = "El usuario no existe?";
            let modal_render = Modal1::new(message).render().unwrap();
            return HttpResponse::Unauthorized().body(modal_render)
        }
    };
    let hashed_pass = user_data.password;
    match verify_password(&form_login.pass,&hashed_pass){
        Ok(res)=>{
            if res{
                //aqui añadir algo en la respuesta, en el header, o añadir la cookie
                //let token_result  = generate_local_token();
                //aqui meter los datos al hashmap nuevo
                let mut claims_data = HashMap::new();
                let new_jti = Uuid::new_v4().to_string();
                claims_data.insert("iss".to_string(),"Molushop.com".to_string());
                claims_data.insert("aud".to_string(),"Molushop/user".to_string());
                claims_data.insert("sub".to_string(),user_data.id.to_string());
                claims_data.insert("jti".to_string(),new_jti.clone());

                let token_result  = generate_local_token_duration(&DurationTime::from_secs(3600),&claims_data);
                match token_result{
                    Ok(token)=>{
                        //crear la cookie --> método?
                        let mut cookie = Cookie::new("sesion_token", token.clone());
                        let mut now = OffsetDateTime::now_utc();
                        now += DurationTime::from_secs(3600);
                        //cookie.set_expires(now);
                        cookie.set_max_age(CookieDuration::seconds(3600));
                        cookie.set_http_only(true);
                        cookie.set_secure(true);
                        cookie.set_path("/");

                        //aquí el login se ha completado
                        //sacar el user_agent
                        let user_agent = match req
                            .headers()
                            .get("User-Agent") // Obtenemos el HeaderValue
                            .and_then(|value| value.to_str().ok())
                            {
                                Some(ua)=>Some(ua),
                                None=>None
                            };
                        
                        //crear los objetos de tiempo
                        let ahora = Utc::now().naive_utc();
                        let expira_en = match ahora.checked_add_signed(Duration::seconds(3600)){
                            Some(time)=>time,
                            None => Utc::now().naive_utc()
                        };
                        let ip_net:IpNet;
                        //obtener la ip
                        let peer_addr = req.peer_addr();
                        let ip_addr = match peer_addr{
                            Some(addr)=>{
                                let ip_addr = addr.ip();
                                ip_net = ip_addr.into();
                                Some(&ip_net)
                            },
                            None=> None
                        };

                        //crear un objeto de inserción
                        let new_user_session = NewUserSession{
                            id:&Uuid::new_v4(),
                            user_id:&user_data.id,
                            jti:&new_jti,
                            refresh_token_hash:&token,
                            issued_at: &ahora,
                            expires_at: &expira_en,
                            last_used_at: &ahora,
                            is_revoked: &false,
                            ip_address: ip_addr,
                            user_agent: user_agent,
                            device_info: Some("por ahora xd"),
                        };
                        //si falla el insert soltar el error
                        match insert_session_login(new_user_session,&pool).await{
                            Ok(_)=>{
                                //let message = "El password es correcto";
                                let modal_render = Modal1::new("El login se ha realizado correctamente").render().unwrap();
                                return HttpResponse::Ok()
                                    .cookie(cookie)
                                    .body(modal_render)
                            },
                            Err(e)=>{
                                println!("Error pete: {}",e);
                                let message = "Fallo ";
                                let modal_render = Modal1::new(message).render().unwrap();
                                return HttpResponse::InternalServerError().body(modal_render)  
                            }
                        }
                    },
                    Err(e)=>{
                        println!("Error pete: {}",e);
                        let message = "Peto en el token";
                        let modal_render = Modal1::new(message).render().unwrap();
                        return HttpResponse::Unauthorized().body(modal_render)   
                    }
                }
            }else {
                let message = "LA CONTRSAEÑA NO ES LA MISMA";
                let modal_render = Modal1::new(message).render().unwrap();
                return HttpResponse::Unauthorized().body(modal_render)    
            }
        },
        Err(e)=>{
            println!("Error! -> {}",e);
            let message = "Ha habido un problema resolviendo el pass";
            let modal_render = Modal1::new(message).render().unwrap();
            return HttpResponse::InternalServerError().body(modal_render)
        }
    }

}

