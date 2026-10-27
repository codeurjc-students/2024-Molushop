//dependecias 
use actix_web::{get, post, web,delete, App, HttpResponse,HttpRequest, HttpServer, Responder, http::StatusCode};
use crate::i18n::{I18n, Lang, LANG_COOKIE};
use lazy_static::lazy_static;
use uuid::Uuid;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;
use askama::Template;
use crate::middleware::auth::Auth;
use crate::constants::urls::{HOME_URL, SEARCH_URL};
//scope ??
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
        .route("/", web::get().to(redirect_home))
        ;
        //.route(format!("{}/{}",DELETE_PRODUCT, "{id}"), web::get().to(products_panel));

}

/// La raíz lleva a la portada. "/en" (y "/en/") a la inglesa; "/" a la del
/// idioma elegido en el selector (cookie) o, si no hay, al que pide el
/// navegador; si no es ninguno de los nuestros, español.
/// 302 y no 301: depende de quién entra, y una 301 se quedaría guardada en
/// navegadores y buscadores.
pub async fn redirect_home(req: HttpRequest) -> HttpResponse {
    let (lang, _) = Lang::split_path(req.path());
    let lang = if lang != Lang::DEFAULT {
        lang
    } else {
        req.cookie(LANG_COOKIE)
            .and_then(|cookie| Lang::from_code(cookie.value()))
            .or_else(|| req.headers()
                .get(actix_web::http::header::ACCEPT_LANGUAGE)
                .and_then(|value| value.to_str().ok())
                .and_then(Lang::from_accept_language))
            .unwrap_or(Lang::DEFAULT)
    };
    HttpResponse::Found()
        .insert_header((actix_web::http::header::LOCATION, format!("{}{}", lang.prefix(), HOME_URL)))
        .insert_header((actix_web::http::header::VARY, "Accept-Language, Cookie"))
        .finish()
}

//el modelo lo creo aqui o en la pagina de modelos
use crate::models::pages_models::home_page_model::*;
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
use crate::services::search_service::{category_counts, children};
use crate::models::components::product_card_group_model::CardFilter;
use crate::models::pages_models::search_page_model::CategoryLink;

/// Los chips de la portada: las categorías de arriba del todo que tienen algo
/// a la venta, cada una con su búsqueda. Si falla, la portada sale sin ellos.
async fn home_categories(lang: Lang, pool: &DbPool) -> Vec<CategoryLink> {
    let categories = match category_counts(&[], lang, pool).await {
        Ok(categories) => categories,
        Err(e) => {
            println!("Home: error cargando las categorías: {}", e);
            return Vec::new();
        }
    };
    let all = CardFilter::new(None, None);
    children(&categories, None)
        .into_iter()
        .map(|c| CategoryLink {
            name: c.name.clone(),
            url: all.with_category(Some(&c.id)).url(SEARCH_URL, None),
            products: c.products,
        })
        .collect()
}

async fn get_home(i18n: I18n, pool_data:web::Data<DbPool>,req: HttpRequest,opt_session_data:Option<web::ReqData<SessionData>>)-> HttpResponse{
    let pool = pool_data.get_ref();
    
    let mut user_name = "".to_string();
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
                user_name = user.username;
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
    
    let home_render = Home{
        user_logged,
        page_name:i18n.t("home-page-title"),
        group_cards2:get_product_card_group_render_all(&i18n, user_id_opt.as_ref(), &pool).await,
        categories:home_categories(i18n.lang, pool).await,
        nav1:get_nav1_object(user_name, user_id_opt.as_ref(), pool).await,
        login_base_data: login_base_service::get_login_base_model_data(),
        i18n: i18n
    }.render().unwrap();
    

    HttpResponse::Ok().body(home_render)

}   