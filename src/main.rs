use dotenv::dotenv;
use oidc::OIDC;
use std::env;

mod oauth2_client;
mod oidc;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let customer_id = env::var("CUSTOMER_ID").expect("CUSTOMER_ID must be set");
    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID must be set");
    let client_secret = env::var("CLIENT_SECRET").expect("CLIENT_SECRET must be set");
    let redirect_uri = env::var("REDIRECT_URI").expect("REDIRECT_URI must be set");
    let citrix_application_id =
        env::var("CITRIX_APPLICATION_ID").expect("CITRIX_APPLICATION_ID must be set");

    let mut oidc_client = OIDC::new(customer_id, citrix_application_id);
    oidc_client.initialize().await;
    let mut oauth2_client = oauth2_client::OAuth2Client::new(
        client_id,
        client_secret,
        redirect_uri,
        oidc_client.authorization_endpoint.clone(),
        oidc_client.token_endpoint.clone(),
        oidc_client.acr_values.clone(),
    );

    oauth2_client.authenticate().await;
    println!("Hello, world!");
}
