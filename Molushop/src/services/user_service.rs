//! Usuarios: comprobar si el correo o el nombre ya existen, buscarlos y darlos de alta.

use diesel::dsl::exists;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::{insert_into, select};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::models::models_x::UserAuth;

type DbPool = Pool<AsyncPgConnection>;

//Obtener si existe un usuario con correo registrado
pub async fn check_mail(mail:&String,pool: &DbPool)->Result<bool,Error>{
    use crate::schema::base_user::dsl::*;

    let connection = &mut pool.get().await.unwrap();

    let result = select(exists(
        base_user.filter(email.eq(mail))
    ))
    .get_result::<bool>(connection)
    .await;

    result
}

pub async fn check_username(user:&String,pool: &DbPool)->Result<bool,Error>{
    use crate::schema::base_user::dsl::*;

    let connection = &mut pool.get().await.unwrap();

    let result = select(exists(
        base_user.filter(username.eq(user))
    ))
    .get_result::<bool>(connection)
    .await;

    result
}

//obtener el usuario y en este caso, su contraseña hasheada
//Que devuelva un objeto con lo que se quiera  obtener del usuario
// Poner la restriccion de que el usuario esté activo
pub async fn get_user_by_username(username_in:&String,pool:&DbPool)->Result<UserAuth,Error>{
    use crate::schema::base_user::dsl::*;
    let connection = &mut pool.get().await.unwrap();

    let user = base_user
        .filter(username.eq(username_in))
        .select((
            id,
            username,
            name,
            lastname,
            email,
            password
        ))
        .first::<UserAuth>(connection)
        .await;

    user
}

pub async fn get_user_by_id(user_id:&Uuid,pool:&DbPool)->Result<UserAuth,Error>{
    use crate::schema::base_user::dsl::*;
    let connection = &mut pool.get().await.unwrap();

    let user = base_user
        .filter(id.eq(user_id))
        .select((
            id,
            username,
            name,
            lastname,
            email,
            password
        ))
        .first::<UserAuth>(connection)
        .await;

    user
}

//pasar modelo base de entrada? --> o un nuevo modelo?
//Se tiene que hashear la contraseña, además 
pub async fn insert_user(username_in:&String,email_in:&String,password_in:&String,pool: &DbPool)->Result<Uuid,Error>{
    use crate::schema::base_user::dsl::*;
    use crate::models::models_x::NewBaseUserSimple;

    let connection = &mut pool.get().await.unwrap();

    let new_id = Uuid::new_v4();

    let user = NewBaseUserSimple{
        id:&new_id,
        username:&username_in,
        email:&email_in,
        password:password_in
    };

    let result = insert_into(base_user).values(user).execute(connection).await;

    match result{
        Ok(_)=>{
            return Ok(new_id)
        },
        Err(e)=>{
            return Err(e)
        }
    }
}
