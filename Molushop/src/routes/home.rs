//dependecias 
use actix_web::{get, post, web,delete, App, HttpResponse,HttpRequest, HttpServer, Responder, http::StatusCode};
use lazy_static::lazy_static;
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;
use rinja::Template;
use crate::middleware::auth::Auth;
//scope ??
//master/es/home
// --> dispatcher --> ---> /home

//cada página tiene su modelo, eso es vrd --> organizarlo tmb
// pub fn scope() -> Scope {
//     web::scope(SCOPE)
//         .configure(config)
// }

pub fn config(cfg: &mut web::ServiceConfig) {
    //cfg.service(products_panel_prueba);
    cfg
        .route(
            "/home", 
            web::get().to(get_home).wrap(Auth::new())
        )
        ;
        //.route(format!("{}/{}",DELETE_PRODUCT, "{id}"), web::get().to(products_panel));

}
//url--> master/es/home

//el modelo lo creo aqui o en la pagina de modelos
use crate::models::pages_models::master::es::home_model::*;
use crate::services::components::{
    product_card_service::*,
    product_card_group_service::*,
    nav1_service::*
};
use crate::services::paseto_token_session_service::*;
use crate::services::servicesX::{check_session,get_user_2};
use crate::models::models_x::UserSession;
use std::collections::HashMap;

use chrono::NaiveDate;
use chrono::prelude::*;

use crate::middleware::auth::SessionData;
use crate::services::components::login_base_service;

async fn get_home(pool_data:web::Data<DbPool>,req: HttpRequest,opt_session_data:Option<web::ReqData<SessionData>>)-> HttpResponse{
    let pool = pool_data.get_ref();
    
    let mut nombre_aux = "".to_string();
    let mut user_logged = false;

    if let Some(req_session_data) = opt_session_data{
        let session_data = req_session_data.into_inner();
        //si el usuario está loggeado, obtener los datos del usuario.
        //llamar al servicio para que me obtenga los datos de los favoritos y
        let user_id = session_data.id; 
                //datos del usuario
        match get_user_2(&user_id,&pool).await{
            Ok(user)=>{
                nombre_aux = user.username;
                user_logged = true;
            }
            Err(e)=>{
                println!("Ha ocurrido un error con la base de datos!");
            }
        }
        
    }else {
        println!("Usuario sin loggear")
    }
    //let user_data = "cosas";


    // aqui renderizas la página
    // y colocas los componentes que quieras 
    //renderizar la pagina HOME
    
    let id = Uuid::parse_str("6b495c7c-a550-4e78-b961-2169e31a6158").unwrap();
    let id2 = Uuid::parse_str("1ec06324-aa8d-45d8-acb2-91b7c32b3954").unwrap();
    let id3 = Uuid::parse_str("45d7a6d2-ecf3-479e-974e-f15bc158810e").unwrap();
    let vector_refs: Vec<&Uuid> = vec![&id, &id2, &id3];

    //hacer el render directamente o 

    let home_render = Home{
        user_logged,
        page_name:"Home".to_string(),
        product:get_product_card_object(&id, &pool).await,
        pcard1:get_product_card_group_render(vector_refs, &pool).await,
        group_cards2:get_product_card_group_render_all(&pool).await,
        nav1:get_nav1_object(nombre_aux),
        login_base_data:login_base_service::get_login_base_model_data()
    }.render().unwrap();
    

    HttpResponse::Ok().body(home_render)

}   