use pasetors::claims::{Claims, ClaimsValidationRules};
use pasetors::keys::SymmetricKey;
use pasetors::{local, Local, version4::V4};
use pasetors::token::UntrustedToken;
use core::convert::TryFrom;
use pasetors::errors::Error;
use std::time::Duration;
use lazy_static::lazy_static;
use std::env;
use std::collections::HashMap;
//cargar del env --> secret key?
//lazy constantes ? --> se debería leer del env solo 1 vez?
lazy_static! { //al ser lazy static se ejecuta una sola vez ya que se reutiliza
    pub static ref LOCAL_SECRET: String = env::var("PASETO_LOCAL_SECRET").unwrap_or("bru".to_string());
    pub static ref SK:SymmetricKey::<V4> = SymmetricKey::<V4>::from(&LOCAL_SECRET.as_bytes()).unwrap();
}

pub fn generate_local_token()->Result<String,Error>{
    let mut claims = Claims::new()?;
    claims.add_additional("data", "A secret, encrypted message")?;
    // Generate the key and encrypt the claims.
    let token = local::encrypt(&SK, &claims, None, Some(b"implicit assertion"))?;
    Ok(token)
}

pub fn generate_local_token_duration(time:&Duration,claims_data:&HashMap<String,String>)->Result<String,Error>{
    let mut claims = Claims::new_expires_in(time)?;
    //claims.add_additional("data", "A secret, encrypted message")?;
    
    claims_mapper(&mut claims,claims_data)?;

    let token = local::encrypt(&SK, &claims, None, Some(b"implicit assertion"))?;
    
    //aqui introducirlo a la base de datos ---> otra función

    Ok(token)
}

pub fn claims_mapper(claims:&mut Claims,claims_data:&HashMap<String,String>) -> Result<(),Error>{
    if claims_data.contains_key("iss"){//Issuer
        let iss = claims_data.get("iss").unwrap();
        claims.issuer(iss)?;
    }
    if claims_data.contains_key("aud"){//Audience
        let aud = claims_data.get("aud").unwrap();
        claims.audience(aud)?;
    }
    if claims_data.contains_key("sub"){//Subject
        let sub = claims_data.get("sub").unwrap();
        claims.subject(sub)?;
    }
    if claims_data.contains_key("jti"){//ID del token
        let jti = claims_data.get("jti").unwrap();
        claims.token_identifier(jti)?;
    }
    Ok(())
}

pub fn claims_validator_mapper(validation_rules:&mut ClaimsValidationRules, claims_data:&HashMap<String,String>)-> Result<(),Error>{
    if claims_data.contains_key("iss"){//Issuer
        let iss = claims_data.get("iss").unwrap();
        validation_rules.validate_issuer_with(iss);
    }
    if claims_data.contains_key("aud"){//Audience
        let aud = claims_data.get("aud").unwrap();
        validation_rules.validate_audience_with(aud);
    }
    if claims_data.contains_key("sub"){//Subject
        let sub = claims_data.get("sub").unwrap();
        validation_rules.validate_subject_with(sub);
    }
    if claims_data.contains_key("jti"){//ID del token
        let jti = claims_data.get("jti").unwrap();
        validation_rules.validate_token_identifier_with(jti);
    }
    Ok(())
}

pub fn validate_local_token(token: &str, claims_data: &HashMap<String, String>) -> Result<String, Error> {
    let mut validation_rules = ClaimsValidationRules::new();
    claims_validator_mapper(&mut validation_rules, claims_data)?;

    let untrusted_token = UntrustedToken::<Local, V4>::try_from(token)?;
    let sk= SymmetricKey::<V4>::from(&LOCAL_SECRET.as_bytes())?;
    //si no se decripta salta un error
    let trusted_token = local::decrypt(&sk, &untrusted_token, &validation_rules, None, Some(b"implicit assertion"))?;

    let claims = trusted_token.payload_claims().unwrap();

    let jti = claims.get_claim("jti").unwrap().as_str().unwrap_or("").to_string();
    //claim.contains_claims --> para verificar si tiene un determinado claim!
    Ok(jti)
    //devolver el jti
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY: &[u8; 32] = b"molushop-test-key-32-bytes-long!";

    /// Token creado con pasetors 0.7.4 (sub = 42, jti = "jti-de-prueba",
    /// caduca en 2100). Si este test falla tras actualizar pasetors, las
    /// sesiones abiertas dejarían de valer: no actualizar sin revisarlo.
    const OLD_TOKEN: &str = "v4.local.OJIOCisivhZjku07VLyAT0WouBFg4QnAbSEeWTuyVg0fdrcIogaZjVh7nCkA1HpVWli21XefEC-I6GWoYMiS2o2C0INzRJJ6-rGosOHjn5sozy6czExbtQEtg72Q-tBfHmxH7dO1gYeHOZw7AQT2c_vr1XcTGYp2AgfDZuNkOcs3vzs7x_t7pmYCV92AWxZWZykyViQ9rJ2NyO74qOdrSlEAzqpou0ARotlrdOTiFRdTe0ohqWuq9pzBs7hM-Q";

    fn decrypt_jti(token: &str, claims_data: &HashMap<String, String>) -> Result<String, Error> {
        let key = SymmetricKey::<V4>::from(TEST_KEY)?;
        let mut rules = ClaimsValidationRules::new();
        claims_validator_mapper(&mut rules, claims_data)?;
        let untrusted = UntrustedToken::<Local, V4>::try_from(token)?;
        let trusted = local::decrypt(&key, &untrusted, &rules, None, Some(b"implicit assertion"))?;
        Ok(trusted.payload_claims().unwrap().get_claim("jti").unwrap().as_str().unwrap().to_string())
    }

    #[test]
    fn token_from_previous_version_still_validates() {
        let data = HashMap::from([("sub".to_string(), "42".to_string())]);
        assert_eq!(decrypt_jti(OLD_TOKEN, &data).unwrap(), "jti-de-prueba");
    }

    #[test]
    fn token_with_wrong_subject_is_rejected() {
        let data = HashMap::from([("sub".to_string(), "43".to_string())]);
        assert!(decrypt_jti(OLD_TOKEN, &data).is_err());
    }

    #[test]
    fn new_token_round_trip() {
        let key = SymmetricKey::<V4>::from(TEST_KEY).unwrap();
        let mut claims = Claims::new_expires_in(&Duration::from_secs(60)).unwrap();
        claims.subject("7").unwrap();
        claims.token_identifier("nuevo").unwrap();
        let token = local::encrypt(&key, &claims, None, Some(b"implicit assertion")).unwrap();
        let data = HashMap::from([("sub".to_string(), "7".to_string())]);
        assert_eq!(decrypt_jti(&token, &data).unwrap(), "nuevo");
    }
}
