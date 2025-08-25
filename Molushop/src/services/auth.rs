use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use crate::models::error::*;

pub fn hash_password(password: &str)-> Result<String, ServiceError>{
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(password.as_bytes(),&salt){
        Ok(hash) => hash.to_string(),
        Err(e)=> return Err(ServiceError::InternalServerError(format!("Error hashing password: {}", e))),
    }; //pasarlo a to_string
    Ok(password_hash)
}

pub fn verify_password(password: &str,hash:&str)-> Result<bool,ServiceError>{
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(ph) => ph,
        Err(e) => return Err(ServiceError::InternalServerError(format!("Error parsing hash: {}", e))),
    };
    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}