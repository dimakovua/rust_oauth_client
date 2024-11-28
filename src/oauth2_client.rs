use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};

pub const RESPONSE_TYPE: &str = "code";
pub const PROMPT: &str = "Login";
pub const SCOPE: &str = "openid wsp spa leases";
pub const CODE_CHALLENGE_METHOD: &str = "S256";
pub const RESPONSE_MODE: &str = "form_post";

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
    ) -> OAuth2Client {
        OAuth2Client {
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

    pub fn authenticate(&self) -> bool {
        if !self.send_authorization_request() {
            println!("Failed to send authorization request");
            return false;
        }
        if !self.retrieve_token() {
            println!("Failed to retrieve token");
            return false;
        }

        true
    }

    pub fn get_access_token(&mut self) -> String {
        self.code_verifier = generate_random_string(128);
        self.code_challenge = sha256_hash(&self.code_verifier);

        let res = String::new();
        res
    }

    pub fn send_authorization_request(&self) -> bool {
        println!("Send Authorization Request");

        true
    }

    pub fn retrieve_token(&self) -> bool {
        println!("Retrieve Token");

        false
    }
}
