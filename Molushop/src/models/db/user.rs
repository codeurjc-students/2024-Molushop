//! Usuarios (`base_user`).

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::schema::*;
use diesel::prelude::*;

#[derive(Insertable)]
#[diesel(table_name = base_user)]
pub struct NewBaseUserSimple<'a> {
    pub id:&'a Uuid,
    pub username:&'a str,
    pub email:&'a str,
    pub password:&'a str
}

#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct UserAuth {
    pub id:Uuid,
    pub username: String,
    pub name: Option<String>,
    pub lastname: Option<String>,
    pub email: String,
    pub password: String,
}
