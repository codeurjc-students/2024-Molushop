use std::sync::OnceLock;

use regex::Regex;

pub fn is_valid_email(email: &str) -> bool {
    // Se compila una vez, no en cada alta
    static EMAIL: OnceLock<Regex> = OnceLock::new();
    EMAIL
        .get_or_init(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9-]+(\.+[a-zA-Z0-9-]{1,})*\.[a-zA-Z]{2,}$").unwrap())
        .is_match(email)
}

/// Al menos 8 caracteres, con minúscula, mayúscula, número y un carácter especial.
pub fn is_password_strong(pass: &str) -> bool {
    // Verifica que la contraseña tenga al menos 8 caracteres
    if pass.len() < 8 {
        return false;
    }
    
    // Verifica que contenga al menos una letra minúscula
    let has_lowercase = pass.chars().any(|c| c.is_ascii_lowercase());
    
    // Verifica que contenga al menos una letra mayúscula
    let has_uppercase = pass.chars().any(|c| c.is_ascii_uppercase());
    
    // Verifica que contenga al menos un dígito
    let has_digit = pass.chars().any(|c| c.is_ascii_digit());
    
    // Verifica que contenga al menos un carácter especial
    let special_chars = "!@#$%^&*()?-_=+{};:,<.>";
    let has_special = pass.chars().any(|c| special_chars.contains(c));
    
    // La contraseña es fuerte si cumple con todos los criterios
    
    has_lowercase && has_uppercase && has_digit && has_special
}