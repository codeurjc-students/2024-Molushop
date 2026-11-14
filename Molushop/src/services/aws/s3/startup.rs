
use crate::client::Client;
use dotenvy::dotenv;
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
    let aws_config_builder = aws_sdk_s3::config::Builder::new()
        .region(aws_region)
        .credentials_provider(aws_cred);

    let aws_config = aws_config_builder.build();
    Client::new(aws_config)
}