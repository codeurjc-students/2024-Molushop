//! Tokens secretos que viajan al usuario (cookie de sesión, enlace de
//! recuperar contraseña). En la base de datos se guarda siempre su hash.

use sha2::{Digest, Sha256};
use uuid::Uuid;

/// SHA-256 en hexadecimal (64 caracteres). Es lo que se guarda del token.
pub fn sha256_hex(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

/// Token aleatorio para un enlace: 64 caracteres hexadecimales, 244 bits de
/// azar del generador del sistema (dos UUID v4; cada uno trae 122 bits
/// aleatorios). Sin dependencias nuevas y seguro para usar en una URL.
pub fn random_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_known_vector() {
        assert_eq!(sha256_hex("abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    #[test]
    fn random_tokens_are_long_and_different() {
        let a = random_token();
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, random_token());
    }
}
