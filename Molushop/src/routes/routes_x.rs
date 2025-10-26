// paginas de la aplicación
use actix_web::{get, post, web,delete, App, HttpResponse,HttpRequest, HttpServer, Responder, http::StatusCode};
use bigdecimal::BigDecimal;
use lazy_static::lazy_static;
use tera::Tera;
use actix_files as fs;
use serde::Deserialize;
use serde::Serialize;
use uuid::timestamp::context;
use std::{str::FromStr, sync::Mutex};
//use crate::database::carrito_numero_productos;
//use crate::database::{self, tiene_productos};
use uuid::Uuid;
use crate::models::data_transfer_objects::product::Product;
use crate::models::get_product::GetProductForm;
use crate::schema::products::variations;
use crate::servicesX::*;
use serde_json::Value;

use crate::models::models_x::*;
//use crate::models::pages::{CategoryTemplate,EditProductTemplate,TemplateEjemplo,ProductsPanelPrueba,Product};
use crate::models::pages::*;

use rinja::Template;

use crate::controllers::createProduct::create_product_controllers::ROUTES;
use crate::controllers::components::product_panel_group::ROUTES as ROUTES_PRODUCT_PANEL_GROUP;

use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

use std::time::Instant; //para medir el tiempo de ejecución de una función
/* 
use rinja::Template;

#[derive(Template,Clone,Debug)]
#[template(path = "category.html")]
struct CategoryTemplate {
    categories: Vec<Category2>,
}
*/



lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza
    pub static ref TEMPLATES: Tera = {
        let source = "templates/**/*";
        let tera = Tera::new(&source).unwrap();
        tera
    };

    //momentaneo
    static ref ID_BASE:Uuid =Uuid::parse_str("95022733-f013-301a-0ada-abc18f151006").unwrap();
    static ref ID_BASE_JOKER:Uuid =Uuid::parse_str("95023733-f013-301a-0ada-abc18f151006").unwrap();

    //static ref CATEGORY_CHILDREN_URL: String = format!("{}/category-children", SCOPE);
    //static ref NEXT_SELECT_URL: String = format!("{}/next-select", SCOPE);
}
static PRODUCT_PANEL_PRUEBA : &str = "/products-panel-prueba";

pub fn config(cfg: &mut web::ServiceConfig) {
    //cfg.service(products_panel_prueba);
    cfg.service(index);
    cfg.service(products_panel_create);
    cfg.service(new_created_product);
    cfg.service(products_panel_edit);
    cfg.service(login1);
    cfg.route(PRODUCT_PANEL_PRUEBA, web::get().to(products_panel_prueba));
    cfg.route("/products-panel", web::get().to(products_panel));

}
use crate::services::products_panel;
use crate::services::components::edit_product;
use crate::controllers::components::edit_product_controller::ROUTES as ROUTES_EDIT_PRODUCT;
use crate::models::components::edit_product::EditProduct;
use crate::models::components::title::Title;

#[get("/products-panel/edit/{id}")]
async fn products_panel_edit(path:web::Path<Uuid>,pool_data:web::Data<DbPool>, req: HttpRequest) -> impl Responder {
    let user_id = Uuid::parse_str("2064d62a-4978-4fe7-bef2-7690ff09bdc8").unwrap(); 
    let product_id = path.into_inner();
    let product_edit_result = edit_product::get_product(&user_id,&product_id,pool_data.get_ref()).await;
    let product_edit = match product_edit_result{
        Ok(product_edit) => product_edit,
        Err(e) => {
            println!("Error loading product: {}", e);
            return HttpResponse::InternalServerError().body("Error loading product");
        }
    };
    let page_content:String;
    let header_htmx = "HX-Request";
    if let Some(value) = req.headers().get(header_htmx) {
        println!("HX-Request: {:?}", value);
        let edit_product = EditProduct{ 
            product: product_edit,
            routes_edit_product: &ROUTES_EDIT_PRODUCT,  
        }.render().unwrap();
        let title_render = Title{
            title:"Edit Product".to_string()
        }.render().unwrap();
        //ahora juntar los dos
        let render_final = format!("{}{}",title_render,edit_product);
        page_content = render_final;
    }else{
        println!("No hay header HX-Request");
        page_content = ProductsPanelEdit{
            user_logged:false,
            product: product_edit,
            routes_edit_product: &ROUTES_EDIT_PRODUCT,  
            page_name:"Edit Product".to_string()
        }.render().unwrap();
    }
    
    //let page_content: String = template_ejeplo.render().unwrap();
    //let page_content: String = TEMPLATES.render("example.html", &context1).unwrap();
    //print!("{}",page_content);
    HttpResponse::Ok().body(page_content)
}





async fn products_panel(pool_data: web::Data<DbPool>) -> impl Responder {
    let now = Instant::now();
    let pool = pool_data.get_ref();
    //vamos a poner un id de prueba
    let user_id = Uuid::parse_str("2064d62a-4978-4fe7-bef2-7690ff09bdc8").unwrap(); 
    println!("HOla");
    //invocar método para gestionar la lógica 
    let products_result= products_panel::get_products(&user_id,pool).await;
    match products_result{
        Ok(products)=>{
            //ahora vamos a renderizar la página
    
            let page_content = ProductsPanel{
                user_logged:false,
                products,
                routes: &ROUTES_PRODUCT_PANEL_GROUP,
                page_name:"Panel Products".to_string()
            }.render().unwrap();
            let elapsed = now.elapsed();
            println!("Elapsed: {:.2?}", elapsed);
            //HttpResponse::Ok().body(page_content)
            HttpResponse::Ok().body(page_content)
        },
        Err(e)=>{
            println!("Error loading products: {}", e);
            return HttpResponse::InternalServerError().body("Error loading products");
        }         
    }
}                           

//#[get("/products-panel-prueba")]
async fn products_panel_prueba() -> impl Responder {
    let pp = ProductsPanelPrueba{};
    let render = pp.render().unwrap();
    
    //let page_content: String = TEMPLATES.render("products_panel.html", &context1).unwrap();
    //print!("{}",page_content);
    HttpResponse::Ok().body(render)
}



#[get("/example")]
async fn example() -> impl Responder {
    let template_ejeplo = TemplateEjemplo{};
    
    let page_content: String = template_ejeplo.render().unwrap();
    //let page_content: String = TEMPLATES.render("example.html", &context1).unwrap();
    //print!("{}",page_content);
    HttpResponse::Ok().body(page_content)
}

#[get("/")]
async fn index() -> impl Responder {
    let context1 = tera::Context::new();
    
    let page_content: String = TEMPLATES.render("base.html", &context1).unwrap();
    //print!("{}",page_content);
    HttpResponse::Ok().body(page_content)
}

#[get("/")]
async fn index2() -> impl Responder {
 
    let context1 = tera::Context::new();
    
    let page_content: String = TEMPLATES.render("test.html", &context1).unwrap();
    //print!("{}",page_content);
    HttpResponse::Ok().body(page_content)
}

#[get("/products-panel/create")]
async fn products_panel_create(pool_data: web::Data<DbPool>) -> impl Responder {
    //obtener vector de datos de la base de datos
    let pool = pool_data.get_ref();
    let categories = obtain_base_categories(pool).await;
    match(categories){
        Ok(categories) => {
            let category_page  = CategoryTemplate::new_2(
                categories,
                &ROUTES
            );
            let render = category_page.render().unwrap();
            HttpResponse::Ok().body(render)
        },
        Err(e) => {
            println!("Error loading categories: {}", e);
            return HttpResponse::InternalServerError().body("Error loading categories");
        }
    }
}


#[get("/products/{product}")]
async fn new_created_product(path:web::Path<Uuid>,pool_data:web::Data<DbPool>) -> impl Responder {
    let pool= pool_data.get_ref(); 
    //let mut context1 = tera::Context::new();
    
    let id_product = path.into_inner();
    //let id_product= Uuid::parse_str(&product).unwrap();
    //print!("{}",product);
    //obtener vector de datos de la base de datos
    let product = get_product(&id_product,pool).await;

    let mut keys:Vec<&String> = Vec::new();
    match(product){
        Ok(product) => {

            let product_form = GetProductForm::new_from_product(product);
            //context1.insert("product",&product);
            //let cosa = product.variations;
            let edit_product = EditProductTemplate{
                product: product_form
            };
            let rendered = edit_product.render().unwrap();
            HttpResponse::Ok().body(rendered)
        },
        Err(e) => {
            println!("Error loading product: {}", e);
            return HttpResponse::InternalServerError().body("Error loading product");
        }
    }
}

#[get("/imagen-prueba")]
async fn imagen_prueba() -> impl Responder {
    let context1 = tera::Context::new();
    
    let page_content: String = TEMPLATES.render("imagen-prueba.html", &context1).unwrap();
    //print!("{}",page_content);
    HttpResponse::Ok().body(page_content)
}
use crate::services::components::login_base_service;
#[get("/login1")]
async fn login1() -> HttpResponse{
    let render_login = Login1{
        user_logged: false,
        page_name:"Login".to_string(),
        login_base_data: login_base_service::get_login_base_model_data()
    }.render().unwrap();
    HttpResponse::Ok().body(render_login)
}