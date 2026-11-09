use argon2::{
    password_hash::{phc::PasswordHash, PasswordHasher, PasswordVerifier},
    Argon2
};
use crate::models::error::*;

/// Hash argon2id con los parámetros por defecto y una sal aleatoria del
/// sistema (la genera argon2 con `getrandom`).
pub fn hash_password(password: &str)-> Result<String, ServiceError>{
    let hash: PasswordHash = Argon2::default()
        .hash_password(password.as_bytes())
        .map_err(|e| ServiceError::InternalServerError(format!("Error hashing password: {}", e)))?;
    Ok(hash.to_string())
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
        // Mismo algoritmo y parámetros que los hashes ya guardados en la BD
        assert!(hash.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"), "{}", hash);
        assert!(verify_password("Una-Clave-1", &hash).unwrap());
    }
}
