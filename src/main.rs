use oidc::OIDC;

mod oauth2_client;
mod oidc;

#[tokio::main]
async fn main() {
    let mut oidc_client = OIDC::new("pegasusdaas".to_string(), "application_id".to_string());
    oidc_client.initialize().await;
    println!("Hello, world!");
}
