use actix_web::{get, post, web,delete, App, HttpResponse,HttpRequest, HttpServer, Responder, http::StatusCode};
use lazy_static::lazy_static;
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;
use askama::Template;

use crate::models::pages_models::master::es::cart_page_model::*;
use crate::services::components::{
    product_card_service::*,
    product_card_group_service::*,
    nav1_service::*
};
use crate::middleware::auth::SessionData;
use crate::services::components::login_base_service;
use crate::services::servicesX::{check_session,get_user_2};

pub fn config(cfg: &mut web::ServiceConfig) {
    //cfg.service(products_panel_prueba);
    cfg
        .service(get_cart)
        ;
        //.route(format!("{}/{}",DELETE_PRODUCT, "{id}"), web::get().to(products_panel));

}

#[get("/cart")]
async fn get_cart(pool_data:web::Data<DbPool>,req: HttpRequest,opt_session_data:Option<web::ReqData<SessionData>>)-> HttpResponse{
    let pool = pool_data.get_ref();
    
    let mut nombre_aux = "".to_string();
    let mut user_logged = false;
    let mut user_id_opt: Option<Uuid> = None;

    if let Some(req_session_data) = opt_session_data{
        let session_data = req_session_data.into_inner();
        //si el usuario está loggeado, obtener los datos del usuario.
        //llamar al servicio para que me obtenga los datos de los favoritos y
        let user_id = session_data.id;
        user_id_opt = Some(user_id);
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

    
    let id = Uuid::parse_str("5b45ee44-faf4-4939-baf5-eca379cea1e9").unwrap();
    let id2 = Uuid::parse_str("69ebb636-70f4-4ebc-973d-411231185365").unwrap();
    let id3 = Uuid::parse_str("c477ee20-0640-4a97-ba8f-5ad077e17324").unwrap();
    let vector_refs: Vec<&Uuid> = vec![&id, &id2, &id3];

    //hacer el render directamente o 

    let home_render = CartPage{
        user_logged,
        page_name:"Home".to_string(),
        nav1:get_nav1_object(nombre_aux, user_id_opt.as_ref(), pool).await,
        login_base_data:login_base_service::get_login_base_model_data()
    }.render().unwrap();

    HttpResponse::Ok().body(home_render)

}   