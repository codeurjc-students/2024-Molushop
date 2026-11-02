//! Sesiones (`user_sessions`).

use chrono::NaiveDateTime;
use diesel::prelude::*;
use ipnet::IpNet;
use uuid::Uuid;
use crate::schema::*;

#[derive(Insertable)]
#[diesel(table_name = user_sessions)]
pub struct NewUserSession<'a> {
    pub id:&'a Uuid,
    pub user_id:&'a Uuid,
    pub jti:&'a str,
    pub refresh_token_hash: &'a str,
    pub issued_at: &'a NaiveDateTime,
    pub expires_at: &'a NaiveDateTime,
    pub last_used_at: &'a NaiveDateTime,
    pub is_revoked: &'a bool,
    pub ip_address: Option<&'a IpNet>,
    pub user_agent: Option<&'a str>,
    pub device_info: Option<&'a str>,
}
#[derive(Queryable, Debug,Identifiable)]
#[diesel(table_name = user_sessions)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub jti: String,
    pub refresh_token_hash: String,
    pub issued_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    pub last_used_at: NaiveDateTime,
    pub is_revoked: bool,
    pub ip_address: Option<ipnet::IpNet>, // Usa ipnet::IpNet
    pub user_agent: Option<String>,
    pub device_info: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
