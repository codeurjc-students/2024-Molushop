//! Usuarios: comprobar si el correo o el nombre ya existen, buscarlos y darlos de alta.

use diesel::dsl::exists;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::{insert_into, select};
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::models::db::{NewBaseUserSimple, UserAuth};
use crate::schema::base_user;
use crate::utils::db::get_connection;

type DbPool = Pool<AsyncPgConnection>;

/// Si ya hay un usuario con ese correo.
pub async fn check_mail(email: &str, pool: &DbPool) -> Result<bool, Error> {
    let connection = &mut get_connection(pool).await?;
    select(exists(base_user::table.filter(base_user::email.eq(email))))
        .get_result::<bool>(connection)
        .await
}

/// Si ya hay un usuario con ese nombre.
pub async fn check_username(username: &str, pool: &DbPool) -> Result<bool, Error> {
    let connection = &mut get_connection(pool).await?;
    select(exists(base_user::table.filter(base_user::username.eq(username))))
        .get_result::<bool>(connection)
        .await
}

/// Lo que hace falta del usuario, con la contraseña hasheada (para el login).
pub async fn get_user_by_username(username: &str, pool: &DbPool) -> Result<UserAuth, Error> {
    let connection = &mut get_connection(pool).await?;
    base_user::table
        .filter(base_user::username.eq(username))
        .select(user_auth_columns())
        .first::<UserAuth>(connection)
        .await
}

pub async fn get_user_by_id(user_id: &Uuid, pool: &DbPool) -> Result<UserAuth, Error> {
    let connection = &mut get_connection(pool).await?;
    base_user::table
        .filter(base_user::id.eq(user_id))
        .select(user_auth_columns())
        .first::<UserAuth>(connection)
        .await
}

/// Las columnas de `UserAuth`, en su orden.
fn user_auth_columns() -> (base_user::id, base_user::username, base_user::name, base_user::lastname, base_user::email, base_user::password) {
    (base_user::id, base_user::username, base_user::name, base_user::lastname, base_user::email, base_user::password)
}

/// Da de alta al usuario y devuelve su id. `password` llega ya hasheada.
pub async fn insert_user(username: &str, email: &str, password: &str, pool: &DbPool) -> Result<Uuid, Error> {
    let connection = &mut get_connection(pool).await?;
    let new_id = Uuid::new_v4();
    let user = NewBaseUserSimple { id: &new_id, username, email, password };
    insert_into(base_user::table).values(user).execute(connection).await?;
    Ok(new_id)
}
