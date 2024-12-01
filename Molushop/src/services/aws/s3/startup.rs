
use crate::client::Client;
use dotenvy::dotenv;
use std::env;

//este codigo 
pub async fn configure_and_return_s3_client() -> Client {
    // S3 configuration and client
    dotenv().ok(); // Esto te carga las variables de entorno del archivo .env
    // Get id and secret key from the environment
    let aws_key = env::var("AWS_ACCESS_KEY_ID").expect("Failed to get AWS key.");
    let aws_key_secret = env::var("AWS_SECRET_ACCESS_KEY").expect("Failed to get AWS secret key.");
    // build the aws cred
    let aws_cred = aws_sdk_s3::config::Credentials::new(
        aws_key,
        aws_key_secret,
        None,
        None,
        "loaded-from-custom-env",
    );
    // build the aws client
    let aws_region = aws_sdk_s3::config::Region::new(
        std::env::var("AWS_REGION").unwrap_or("eu-west-2".to_string()),
    );
    let aws_config_builder = aws_sdk_s3::config::Builder::new()
        .region(aws_region)
        .credentials_provider(aws_cred);

    let aws_config = aws_config_builder.build();
    Client::new(aws_config)
}
/* 
async fn run(
    listener: std::net::TcpListener,
    db_pool: sqlx::postgres::PgPool,
    settings: crate::settings::Settings,
) -> Result<actix_web::dev::Server, std::io::Error> {
    // For S3 client: create singleton S3 client
    let s3_client = actix_web::web::Data::new(configure_and_return_s3_client().await);
    
    // S3 client
    .app_data(s3_client.clone())
    
}
    */