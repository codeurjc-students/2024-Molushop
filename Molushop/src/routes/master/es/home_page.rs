//dependecias 
use actix_web::{get, post, web,delete, App, HttpResponse,HttpRequest, HttpServer, Responder, http::StatusCode};
use lazy_static::lazy_static;
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;
use rinja::Template;
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
        .service(get_home)
        ;
        //.route(format!("{}/{}",DELETE_PRODUCT, "{id}"), web::get().to(products_panel));

}
//url--> master/es/home

//el modelo lo creo aqui o en la pagina de modelos
use crate::models::pages_models::master::es::home_model::*;
use crate::services::components::{
    product_card_service::*,
    product_card_group_service::*,
};
use crate::services::paseto_token_session_service::*;
use crate::services::servicesX::{check_session,get_user_2};
use crate::models::models_x::UserSession;
use std::collections::HashMap;

use chrono::NaiveDate;
use chrono::prelude::*;

#[get("/home")]
async fn get_home(pool_data:web::Data<DbPool>,req: HttpRequest)-> HttpResponse{
    let pool = pool_data.get_ref();
    let mut claims_data = HashMap::new();
    claims_data.insert("iss".to_string(),"Molushop.com".to_string());
    claims_data.insert("aud".to_string(),"Molushop/user".to_string());
    //verificar que la request tiene la cookie que queremos
    let cookie1 = req.cookie("sesion_token");
    let mut user_logged = false;
    let mut user_id = "".to_string();
    //if cookie1.is_some(){
    if let Some(c) = cookie1{
        let token = c.value().to_string();
        println!("la cookie existe!, su valor es:{}",&token);
        //hacer la validación de la cookie
        
        match validate_local_token(&token,&claims_data){
            Ok(jti)=>{
                if jti != "".to_string(){
                    println!("El uuid es: {}",jti);
                    match check_session(&jti,&pool).await{
                        Ok(us)=>{
                            //verificar que: no esté revoked
                            // esté dentro de los plazos
                            let now = Utc::now().naive_utc();
                            if us.is_revoked{
                                println!("Token ha sido invalidado!");
                            }else if !(now >= us.issued_at && now<=us.expires_at){
                                println!("El token ha expirado!");
                                //ver si aquí ponemos que se revoque el token
                            }else{
                                user_logged=true;
                                user_id=us.user_id.to_string();
                            }
                        },
                        Err(e)=>{
                            // de momento un print --> 
                            println!("Peto SQL!--> {}",e); 
                        }
                    }
                    //user_logged=true;
                    //poner aqui la función de  verificación de sesión de la base de datos
                }
            },
            Err(e)=>{
               println!("Peto!--> {}",e); 
            }
        }
        // en funcioón de si el token es normal --> no devuelva nada especial
    }else {
        println!("la cookie no existe!")
        
    }
    if user_logged{
        //si el usuario está loggeado, obtener los datos del usuario.
        //llamar al servicio para que me obtenga los datos de los favoritos y 
        match Uuid::parse_str(&user_id) {
            Ok(user_id_u) => {
                //datos del usuario
                match(get_user_2(&user_id_u,&pool).await){
                    Ok(user)=>{

                    }
                    Err(e)=>{

                    }
                }
                //datos de carrito

                //datos de los likes

                println!("UUID válido parseado: {}", user_id_u);
            }
            Err(e) => {
                println!("Error al parsear UUID válido: {}", e);
            }
        }
    }else {
        println!("Usuario sin loggear")
    }
    //let user_data = "cosas";


    // aqui renderizas la página
    // y colocas los componentes que quieras 
    //renderizar la pagina HOME
    
    let id = Uuid::parse_str("5b45ee44-faf4-4939-baf5-eca379cea1e9").unwrap();
    let id2 = Uuid::parse_str("69ebb636-70f4-4ebc-973d-411231185365").unwrap();
    let id3 = Uuid::parse_str("c477ee20-0640-4a97-ba8f-5ad077e17324").unwrap();
    let vector_refs: Vec<&Uuid> = vec![&id, &id2, &id3];

    //hacer el render directamente o 

    let home_render = Home{
        page_name:"Home".to_string(),
        product:get_product_card_object(&id, &pool).await,
        pcard1:get_product_card_group_render(vector_refs, &pool).await,
    }.render().unwrap();

    HttpResponse::Ok().body(home_render)

}   