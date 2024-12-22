// logica de las llamadas al server
use actix_web::{get, post, web,delete, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use aws_config::imds::client;
use bigdecimal::BigDecimal;
use lazy_static::lazy_static;
use tera::Tera;
use actix_files as fs;
use serde::Deserialize;
use serde::Serialize;
use std::{str::FromStr, sync::Mutex};
use std::sync::Arc;

//importante importar Template
use rinja::Template;

use actix_multipart::{
    form::{
        tempfile::{TempFile, TempFileConfig},
        MultipartForm,
        json::Json as MpJson
    },
    Multipart,
};

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(limit = "100MB")]
    file: TempFile,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    name: String,
}


use uuid::Uuid;
use crate::models::models_x::Products;
use crate::services::servicesX::*;
use crate::models::models_x::ProductForm;

use crate::services::aws::s3::client::Client;
use crate::services::aws::s3::upload::UploadedFile;

use std::any::type_name;

//models
use crate::models::models_x::Category;
//use crate::models::htmx::create_product::{List_category_base,Base_category,BaseProducto,VariationsInput,VariationsInputExtra,SpecsInput};
use crate::models::htmx::create_product::*;
use crate::models::extra::BaseSpecs;


fn print_type_of<T>(_: &T) {
    println!("El tipo de dato es: {}", type_name::<T>());
}

lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza
    /* 
    pub static ref TEMPLATES: Tera = {
        let source = "templates/**/*";
        let tera = Tera::new(&source).unwrap();
        tera
    };
    */    

    //momentaneo
    static ref ID_BASE:Uuid =Uuid::parse_str("95022733-f013-301a-0ada-abc18f151006").unwrap();
    static ref ID_BASE_JOKER:Uuid =Uuid::parse_str("95023733-f013-301a-0ada-abc18f151006").unwrap();

    //static ref TASK_COUNTER: Mutex<i32> = Mutex::new(0);
    //static ref NEXT_ID: AtomicI32 = AtomicI32::new(1);
}

#[get("/prueba-insertar")]
async fn prueba_insertar() -> impl Responder {
    
    if(insert_data_test()){
        HttpResponse::Ok().body("Datos insertados")
    }else{
        HttpResponse::Ok().body("Error al insertar datos")
    }

    //HttpResponse::Ok().body(database::total_carrito(&ID_BASE).to_string()+" €")
}

#[get("/prueba-modificar")]
async fn prueba_modificar() -> impl Responder {
    
    if(modify_data_test()){
        HttpResponse::Ok().body("Datos modificados")
    }else{
        HttpResponse::Ok().body("Error al modificar datos")
    }

    //HttpResponse::Ok().body(database::total_carrito(&ID_BASE).to_string()+" €")
}
/*
#[get("/category-children/{category_id}")]
async fn category_children(path: web::Path<String>) -> impl Responder {
    let category_id= path.into_inner();

    let categories_result = obtain_categories_children(&category_id);
    let ancestors_result: Result<Vec<crate::models::models_x::Category>, diesel::result::Error> = obtain_ancestors(&category_id);
    match (categories_result, ancestors_result) {
        (Ok(categories), Ok(ancestors)) => {


            let list_category_base = List_category_base::new_2(ancestors,categories);
            let rendered = list_category_base.render().unwrap();
            HttpResponse::Ok().body(rendered)
        },
        (Err(e), Ok(_)) => {
            println!("Error loading categories: {}", e);
            return HttpResponse::InternalServerError().body("Error loading categories");
        },
        (Ok(_), Err(e)) => {
            println!("Error loading ancestors: {}", e);
            return HttpResponse::InternalServerError().body("Error loading ancestors");
        },
        (Err(e1), Err(e2)) => {
            println!("Error loading categories: {}", e1);
            println!("Error loading ancestors: {}", e2);
            return HttpResponse::InternalServerError().body("Error loading categories and ancestors");
        }
    }
}
*/
/*
#[get("/reset-category")]
async fn reset_category() -> impl Responder {
    let categories = obtain_base_categories();
    match categories{
        Ok(categories) => {
            let base_category = Base_category::new(categories); 
            let rendered = base_category.render().unwrap();
            HttpResponse::Ok().body(rendered)
        },
        Err(e) => {
            println!("Error loading categories: {}", e);
            return HttpResponse::InternalServerError().body("Error loading categories");
        }
    }
}
 
#[get("/next-select/{category_id}")]
async fn next_category(path: web::Path<String>) -> impl Responder {
    let category_id= path.into_inner();
    //vamos a obtener el json de la plantilla de la categoría
    let specs_result= obtain_base_specs(&category_id);
    match specs_result {
        Ok(specs) => {
            let first = specs.first().unwrap();
            match(first){
                Some(spec) => {
                    let base_specs = serde_json::from_value(spec.clone()).unwrap();
                    let base_product = BaseProducto::new_2(base_specs,category_id);       
                    let render  = base_product.render().unwrap();
                    
                    return HttpResponse::Ok().body(render)
                },
                None => {
                    println!("Error loading specs");
                    return HttpResponse::InternalServerError().body("Error loading specs");
                }
            }
        },
        Err(e) => {
            println!("Error loading specs: {}", e);
            return HttpResponse::InternalServerError().body("Error loading specs");
        }
    }
}
*/
#[post("/create-product/{category_id}")]
async fn create_product(path:web::Path<String>,data: web::Json<ProductForm>) -> impl Responder {
    let category_id= path.into_inner();
    println!("{:?}",category_id);
    
    //print_type_of(&data);
    //hacer print del tipo que es data
    let datox = data.into_inner();
    println!("{:?}",datox);
    match insert_new_product(&datox){
        Ok(_) => {
            println!("Producto insertado");
            HttpResponse::Ok().body("Producto creado")
        },
        Err(e) => {
            println!("Error al insertar producto: {}", e);
            return HttpResponse::InternalServerError().body("Error al insertar producto");
        }
    }
}

// #[get("/add-variations")]
// async fn add_variation() -> impl Responder {
//     //let context = tera::Context::new();
//     let variations_input = VariationsInput::new();
//     let rendered = variations_input.render().unwrap();
//     HttpResponse::Ok().body(rendered)
// }

#[get("/add-variations-attributes")]
async fn add_variation_value() -> impl Responder {
    // let context = tera::Context::new();
    //let rendered = TEMPLATES.render("create_product/variations-input-extra.html", &context).unwrap();
    let variations_input_extra = VariationsInputExtra::new();
    let rendered = variations_input_extra.render().unwrap();
    HttpResponse::Ok()
        .insert_header(("HX-Trigger","update_num"))
        .body(rendered)
}

#[get("/add-specs")]
async fn add_specs() -> impl Responder {
    let specs_input = SpecsInput::new();
    let rendered = specs_input.render().unwrap();
    
    HttpResponse::Ok().body(rendered)
}

#[post("/prueba-aws")]
async fn save_files(
    client_data: web::Data<Arc<Client>>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> impl Responder {
    println!("Vamos a ver si se sube el archivo!");
    let client_data_x = client_data.get_ref().as_ref();
    let uploaded_file=  client_data_x.upload(&form.file, "").await;
    println!("Archivo subido!");

    HttpResponse::Ok().body(format!("File uploaded: {:?}", uploaded_file))
}

#[derive(Deserialize)]
struct Delete {
    key: String,
}


#[post("/delete-aws")]
async fn delete_files(
    client_data: web::Data<Arc<Client>>,
    web::Form(form): web::Form<Delete>
) -> impl Responder {
    println!("Vamos a borrar un archivo!");
    let client_data_x = client_data.get_ref().as_ref();
    
    let end =  client_data_x.delete_file(&form.key).await;
    let result = if end {"Archivo borrado!"} else {"Error al borrar el archivo"};

    HttpResponse::Ok().body(result)
}


#[get("/delete-all")]
async fn delete_all_files_2(
    client_data: web::Data<Arc<Client>>
) -> impl Responder {
    println!("Vamos a borrar un archivo!");
    let client_data_x = client_data.get_ref().as_ref();
    let end =  client_data_x.delete_all_files().await;
    match end {
        Ok(_) => {
            println!("Archivos borrados!");
            return HttpResponse::Ok().body("Archivos borrados");
        },
        Err(e) => {
            println!("Error al borrar archivos: {}", e);
            return HttpResponse::InternalServerError().body("Error al borrar archivos");
        }
    }
}



#[get("/delete-fail")]
async fn delete_fail(
    client_data: web::Data<Arc<Client>>
) -> impl Responder{
    let client_data_x = client_data.get_ref().as_ref();
    //let lista = vec!["hola".to_string(),"adios".to_string()];
    let lista = vec!["1733000369bone.jpg".to_string(),"xd".to_string()];

    let end =  client_data_x.delete_files_from_vec(lista).await;
    match end {
        Ok(_) => {
            println!("Archivos borrados!");
            return HttpResponse::Ok().body("Archivos borrados");
        },
        Err(e) => {
            println!("Error al borrar archivos: {}", e);
            return HttpResponse::InternalServerError().body("Error al borrar archivos");
        }
    }
}

#[get("/listar-archivos-s3")]
async fn listar_archivos_s3(
    client_data: web::Data<Arc<Client>>
) -> impl Responder {
    println!("Vamos a listar los archivos!");
    let client_data_x = client_data.get_ref().as_ref();
    //let uploaded_file=  client_data_x.upload(&form.file, "").await;
    let end =  client_data_x.list_objects().await;
    //let result = if end {"Archivos listados!"} else {"Error al listar los archivos"};
    println!("Archivos: {:?}",end);

    HttpResponse::Ok().body("Archivos listados")
}