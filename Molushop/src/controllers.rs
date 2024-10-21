// logica de las llamadas al server
use actix_web::{get, post, web,delete, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use bigdecimal::BigDecimal;
use lazy_static::lazy_static;
use tera::Tera;
use actix_files as fs;
use serde::Deserialize;
use serde::Serialize;
use std::{str::FromStr, sync::Mutex};
//use crate::database::carrito_numero_productos;
//use crate::database::{self, tiene_productos};
use uuid::Uuid;
use crate::services::*;


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