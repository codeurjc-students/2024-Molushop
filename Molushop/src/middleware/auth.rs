use std::future::{ready, Ready};
use std::rc::Rc;
use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, web, Error, HttpMessage
};
use futures_util::future::LocalBoxFuture;
use uuid::Uuid;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;
use crate::services::paseto_token_session_service::*;
use crate::services::servicesX::{check_session,get_user_2};
use std::collections::HashMap;
use chrono::prelude::*;
// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
#[derive(Clone)]
pub struct Auth{
    pool_data: web::Data<DbPool>
}

impl Auth{
    pub fn new(pool_data:web::Data<DbPool>)->Self{
        Self {pool_data}
    }
}

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S: 'static, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware { 
            service:Rc::new(service),
            pool_data:self.pool_data.clone() 
        }))
    }
}
#[derive(Clone)]
pub struct AuthMiddleware<S> {
    service: Rc<S>,
    pool_data:web::Data<DbPool>
}

#[derive(Debug,Clone)]
pub struct SessionData{
    pub id : Uuid
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {

        let pool = self.pool_data.get_ref().clone();
        println!("Hi from start. You requested: {}", req.path());

        //puedo llamar a una función que me haga todo lo que tengo que hacer
        //let fut = self.service.call(req);
        //let service = self.service.clone();
        let service = self.service.clone();

        Box::pin(async move {
            let data_send = validate_cookie(&pool,&req).await;

            if let Some(data_session) = data_send{
                
                req.extensions_mut().insert(data_session);
            }

            //let res = fut.await?;
            let res = service.call(req).await?;

            println!("Hi from response");
            Ok(res)
        })
    }
}

async fn validate_cookie(pool:&DbPool,req:&ServiceRequest)->Option<SessionData>{
    //ver si tiene una cookie ,
    let mut claims_data = HashMap::new();
    claims_data.insert("iss".to_string(),"Molushop.com".to_string());
    claims_data.insert("aud".to_string(),"Molushop/user".to_string());
    //verificar que la request tiene la cookie que queremos
    let cookie1 = req.cookie("sesion_token");
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
                                //user_id=us.user_id.to_string();
                                return Some(SessionData{id:us.user_id.clone()})
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
        println!("la cookie no existe!");
    }
    return None
}