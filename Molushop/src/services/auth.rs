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

#[cfg(test)]
mod tests {
    use super::*;

    /// El hash de `seeds/dev.sql` se generó fuera de la app (argon2id con los
    /// parámetros por defecto). Si este test falla, los usuarios del seed no
    /// pueden entrar: regenerar el hash del seed.
    #[test]
    fn seed_password_hash_verifies() {
        let seed_hash = "$argon2id$v=19$m=19456,t=2,p=1$FPFBTJ8GSMBbd/ALMKRKgw$Fm6+Zxh3IWC3i8nV4vPYigE9KuwJbIwZBR8Du1G8Ap0";
        assert!(verify_password("Molushop-dev1", seed_hash).unwrap());
        assert!(!verify_password("otra", seed_hash).unwrap());
    }

    #[test]
    fn hash_then_verify() {
        let hash = hash_password("Una-Clave-1").unwrap();
        assert!(verify_password("Una-Clave-1", &hash).unwrap());
    }
}
