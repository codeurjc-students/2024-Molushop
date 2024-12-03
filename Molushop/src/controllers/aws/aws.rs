use super::s3::aws_s3;
use actix_web::Scope;
use actix_web::{get, post, web,delete, App, HttpResponse, HttpServer, Responder, http::StatusCode};



pub fn scope_aws() -> Scope {
    web::scope("/aws")
        .service(aws_s3::scope_s3())
}