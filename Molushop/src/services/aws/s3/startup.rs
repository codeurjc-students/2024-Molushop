
use crate::client::Client;
use dotenvy::dotenv;
use crate::app_env::AppEnv;
use std::env;

//este codigo 
pub async fn configure_and_return_s3_client() -> Client {
    // S3 configuration and client
    dotenv().ok(); // Esto te carga las variables de entorno del archivo .env
    // Get id and secret key from the environment
    // Las cuatro variables de AWS ya se comprobaron al arrancar (utils::required_env).
    let aws_key = env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID is checked at startup");
    let aws_key_secret = env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY is checked at startup");
    // build the aws cred
    let aws_cred = aws_sdk_s3::config::Credentials::new(
        aws_key,
        aws_key_secret,
        None,
        None,
        "loaded-from-custom-env",
    );
    // build the aws client. Sin región por defecto: la de antes (eu-west-2) no era
    // la del bucket, y las URL de las imágenes salían mal.
    let aws_region = aws_sdk_s3::config::Region::new(
        env::var("AWS_REGION").expect("AWS_REGION is checked at startup"),
    );
    let mut aws_config_builder = aws_sdk_s3::config::Builder::new()
        .region(aws_region)
        .credentials_provider(aws_cred);
    // S3 simulado en local (Garage): otra dirección y el bucket en la ruta
    // (`host/bucket/clave`), no en el nombre del host como en AWS. Sin la variable,
    // el S3 de AWS de siempre (Render).
    if let Some(endpoint) = endpoint_url() {
        aws_config_builder = aws_config_builder.endpoint_url(endpoint).force_path_style(true);
    }

    let aws_config = aws_config_builder.build();
    Client::new(aws_config)
}

/// `S3_ENDPOINT_URL` si está y no está vacía.
fn endpoint_url() -> Option<String> {
    env::var("S3_ENDPOINT_URL").ok().filter(|url| !url.trim().is_empty())
}

/// Una línea al arrancar con el S3 que se usa: Garage (u otro, con su dirección)
/// o el de AWS. En `dev` contra AWS es un `warn`: se puede hacer a propósito,
/// pero así se ve enseguida si ha sido sin querer.
pub fn log_s3_target(app_env: AppEnv) {
    let bucket = env::var("AWS_S3_BUCKET_NAME").unwrap_or_default();
    let region = env::var("AWS_REGION").unwrap_or_default();
    match endpoint_url() {
        Some(endpoint) => tracing::info!(%endpoint, %bucket, %region, "S3 storage ready"),
        None if is_production_s3_from_dev(app_env, None) => tracing::warn!(
            endpoint = "aws", %bucket, %region,
            "using the production AWS S3 from dev"
        ),
        None => tracing::info!(endpoint = "aws", %bucket, %region, "S3 storage ready"),
    }
}

fn is_production_s3_from_dev(app_env: AppEnv, endpoint: Option<&str>) -> bool {
    app_env == AppEnv::Dev && endpoint.is_none_or(|url| url.trim().is_empty())
}

/// Con `S3_ENDPOINT_URL` hace falta `S3_PUBLIC_BASE_URL`: si no, las imágenes se
/// subirían al S3 simulado y sus URL apuntarían al de AWS.
pub fn check_s3_config() -> Result<(), String> {
    endpoint_pair_problem(
        env::var("S3_ENDPOINT_URL").ok().as_deref(),
        env::var("S3_PUBLIC_BASE_URL").ok().as_deref(),
    )
}

fn endpoint_pair_problem(endpoint: Option<&str>, public_base: Option<&str>) -> Result<(), String> {
    let is_set = |value: Option<&str>| value.is_some_and(|v| !v.trim().is_empty());
    if is_set(endpoint) && !is_set(public_base) {
        return Err("S3_PUBLIC_BASE_URL must be set when S3_ENDPOINT_URL is set".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{endpoint_pair_problem, is_production_s3_from_dev};
    use crate::app_env::AppEnv;

    #[test]
    fn warns_only_for_aws_from_dev() {
        assert!(is_production_s3_from_dev(AppEnv::Dev, None));
        assert!(is_production_s3_from_dev(AppEnv::Dev, Some(" ")));
        assert!(!is_production_s3_from_dev(AppEnv::Dev, Some("http://127.0.0.1:3900")));
        assert!(!is_production_s3_from_dev(AppEnv::Pre, None));
        assert!(!is_production_s3_from_dev(AppEnv::Prod, None));
    }

    #[test]
    fn aws_without_endpoint_is_fine() {
        assert!(endpoint_pair_problem(None, None).is_ok());
        assert!(endpoint_pair_problem(Some(""), None).is_ok());
    }

    #[test]
    fn endpoint_needs_a_public_base() {
        assert!(endpoint_pair_problem(Some("http://127.0.0.1:3900"), None).is_err());
        assert!(endpoint_pair_problem(Some("http://127.0.0.1:3900"), Some(" ")).is_err());
        assert!(endpoint_pair_problem(Some("http://127.0.0.1:3900"), Some("http://b.web.localhost:3902")).is_ok());
    }
}
