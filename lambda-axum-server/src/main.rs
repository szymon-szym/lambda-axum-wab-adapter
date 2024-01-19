pub mod config;
pub mod http;

use axum::Router;
use clap::Parser;
use config::Config;


#[tokio::main]
async fn main() {

    dotenv::dotenv().ok();

    let config = Config::parse();

    println!("{:?}", config);

    let aws_config = 
        match config.aws_profile {
            Some(profile) => aws_config::from_env().profile_name(profile).load().await,
            None => aws_config::from_env().load().await,
        };

    let dynamodb_client = aws_sdk_dynamodb::Client::new(&aws_config);

    println!("dynamo client initialized");

    let app = Router::new()
        .merge(http::books::router(config.dynamo_table_name, dynamodb_client));
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}