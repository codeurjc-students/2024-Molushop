use pasetors::claims::{Claims, ClaimsValidationRules};
use pasetors::keys::{Generate, SymmetricKey};
use pasetors::{local, Local, version4::V4};
use pasetors::token::UntrustedToken;
use core::convert::TryFrom;
use pasetors::errors::Error;
use std::time::Duration;
use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::env;
use uuid::Uuid;
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
    //println!("{}",&*LOCAL_SECRET);
    // Generate the key and encrypt the claims.
    //let sk= SymmetricKey::<V4>::from("bWkQ3pDwtMhdfzcNEGyUA6sx2nqL5vHV".as_bytes())?;
    println!("hola");
    let token = local::encrypt(&SK, &claims, None, Some(b"implicit assertion"))?;
    Ok(token)
}

pub fn generate_local_token_duration(time:&Duration,claims_data:&HashMap<String,String>)->Result<String,Error>{
    let mut claims = Claims::new_expires_in(time)?;
    //claims.add_additional("data", "A secret, encrypted message")?;
    
    claims_mapper(&mut claims,claims_data)?;

    println!("{:?}",claims);

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

pub fn validate_local_token(token:&String,claims_data:&HashMap<String,String>)->Result<String,Error>{
    let mut validation_rules = ClaimsValidationRules::new();
    claims_validator_mapper(&mut validation_rules, claims_data)?;

    let untrusted_token = UntrustedToken::<Local, V4>::try_from(token)?;
    let sk= SymmetricKey::<V4>::from(&LOCAL_SECRET.as_bytes())?;
    //si no se decripta salta un error
    let trusted_token = local::decrypt(&sk, &untrusted_token, &validation_rules, None, Some(b"implicit assertion"))?;
    //assert_eq!(&claims, trusted_token.payload_claims().unwrap());

    let claims = trusted_token.payload_claims().unwrap();

    let jti = claims.get_claim("jti").unwrap().as_str().unwrap_or("").to_string();
        /* 
        {
        Ok(value)=>{
            value
        },
        None=>"".to_string()
        
    };*/
    println!("{:?}",claims);
    //claim.contains_claims --> para verificar si tiene un determinado claim!
    Ok(jti)
    //devolver el jti
}