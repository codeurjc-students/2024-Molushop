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
    nav1_service::*,
    cart_get_service::*
};
use crate::middleware::auth::{Auth, SessionData};
use crate::services::components::login_base_service;
use crate::services::servicesX::{check_session,get_user_2};

pub fn config(cfg: &mut web::ServiceConfig) {
    //cfg.service(products_panel_prueba);
    cfg
        .route(
            "/cart",
            web::get().to(get_cart).wrap(Auth::new())
        )
        ;
        //.route(format!("{}/{}",DELETE_PRODUCT, "{id}"), web::get().to(products_panel));

}

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

    
    let cart_render = CartPage{
        user_logged,
        page_name:"Cesta".to_string(),
        nav1:get_nav1_object(nombre_aux, user_id_opt.as_ref(), pool).await,
        login_base_data:login_base_service::get_login_base_model_data(),
        cart_total:get_cart_total_object(user_id_opt.as_ref(), pool).await
    }.render().unwrap();

    HttpResponse::Ok().body(cart_render)

}
