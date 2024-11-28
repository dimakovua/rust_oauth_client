use rand::{distributions::Alphanumeric, Rng};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::Shutdown;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::sync::oneshot;
use warp::filters::header;
use warp::Filter;

pub const RESPONSE_TYPE: &str = "code";
pub const PROMPT: &str = "Login";
pub const SCOPE: &str = "openid wsp spa leases";
pub const CODE_CHALLENGE_METHOD: &str = "S256";
pub const RESPONSE_MODE: &str = "form_post";

#[derive(Error, Debug)]
pub enum OAuthError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Status is not OK")]
    NotOk,
    #[error("Failed to parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Failed to get code")]
    CodeErr,
}

fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

fn sha256_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);

    let result = hasher.finalize();
    format!("{:x}", result)
}

async fn start_server(
    code_storage: Arc<Mutex<Option<String>>>,
    shutdown_signal: oneshot::Receiver<()>,
) {
    let code_route = warp::get()
        .and(warp::path::end())
        .and(warp::query::<HashMap<String, String>>())
        .map(move |params: HashMap<String, String>| {
            if let Some(code) = params.get("code") {
                println!("Received code: {}", code);
                *code_storage.lock().unwrap() = Some(code.clone());
                format!("Received code: {}", code)
            } else {
                "Missing code parameter".to_string()
            }
        });

    let (_, server) =
        warp::serve(code_route).bind_with_graceful_shutdown(([127, 0, 0, 1], 8080), async {
            shutdown_signal.await.ok();
        });

    println!("Server started at http://localhost:8080");
    server.await;
}

pub struct OAuth2Client {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    authorization_endpoint: String,
    token_endpoint: String,
    acr_values: String,

    code_verifier: String,
    code_challenge: String,
    authorization_code: String,

    access_token: String,
}

impl OAuth2Client {
    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
        authorization_endpoint: String,
        token_endpoint: String,
        acr_values: String,
    ) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            authorization_endpoint,
            token_endpoint,
            acr_values,
            code_verifier: String::new(),
            code_challenge: String::new(),
            authorization_code: String::new(),
            access_token: String::new(),
        }
    }

    pub async fn authenticate(&mut self) -> bool {
        if let Err(err) = self.send_authorization_request().await {
            println!("Failed to send authorization request {}", err);
            return false;
        }
        if let Err(err) = self.retrieve_token() {
            println!("Failed to retrieve token {}", err);
            return false;
        }

        true
    }

    pub fn get_access_token(&mut self) -> String {
        let res = String::new();
        res
    }

    pub async fn send_authorization_request(&mut self) -> Result<(), OAuthError> {
        self.code_verifier = generate_random_string(128);
        self.code_challenge = sha256_hash(&self.code_verifier);

        let client = reqwest::Client::new();
        let code_storage = Arc::new(Mutex::new(None));
        let code_storage_server = Arc::clone(&code_storage);
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move { start_server(code_storage, rx).await });

        let response = client
            .get(&self.authorization_endpoint)
            .query(&[
                ("response_type", RESPONSE_TYPE),
                ("client_id", &self.client_id),
                ("redirect_uri", &self.redirect_uri),
                ("acr_values", &self.acr_values),
                ("prompt", PROMPT),
                ("scope", SCOPE),
                ("code_challenge", &self.code_challenge),
                ("code_challenge_method", CODE_CHALLENGE_METHOD),
                ("response_mode", RESPONSE_MODE),
                ("state", "123456"),
            ])
            .send()
            .await?;

        let status = response.status();
        if status != reqwest::StatusCode::OK {
            return Err(OAuthError::NotOk);
        }

        if let Some(location_header) = response.headers().get("Location") {
            match location_header.to_str() {
                Ok(redirect_url) => {
                    println!("Redirect URL: {}", redirect_url);
                    match opener::open(redirect_url) {
                        Ok(_) => println!("Opened URL in browser: {}", redirect_url),
                        Err(err) => eprintln!("Failed to open URL: {}", err),
                    }
                }
                Err(err) => {
                    println!("Failed to parse Location header: {}", err);
                }
            }
        }

        let retrieved_code = code_storage_server.lock().unwrap().clone();
        let _ = tx.send(());

        if let Some(code) = retrieved_code {
            self.authorization_code = code;
            println!("Received code: {}", self.authorization_code);
            Ok(())
        } else {
            Err(OAuthError::CodeErr)
        }
    }

    pub fn retrieve_token(&self) -> Result<(), OAuthError> {
        todo!();
    }
}
