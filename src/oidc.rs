#![allow(clippy::upper_case_acronyms)]

use serde_json::Value;
use thiserror::Error;

const CLOUD: &str = "cloud";

#[derive(Error, Debug)]
pub enum OIDCError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Status is not OK")]
    NotOk,
    #[error("Failed to parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),
}

#[derive(Debug)]
pub struct OIDC {
    customer_id: String,
    application_id: String,

    pub acr_values: String,
    encoded_acr_values: String,
    pub oidc_discovery_endpoint: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
}

impl OIDC {
    pub fn new(customer_id: String, application_id: String) -> Self {
        Self {
            customer_id,
            application_id,
            acr_values: String::new(),
            encoded_acr_values: String::new(),
            oidc_discovery_endpoint: String::new(),
            authorization_endpoint: String::new(),
            token_endpoint: String::new(),
        }
    }

    pub async fn initialize(&mut self) -> bool {
        if let Err(err) = self.get_oidc_discovery_endpoint().await {
            println!("Failed to get OIDC discovery endpoint: {}", err);
            return false;
        }

        if let Err(err) = self.get_open_id_configuration().await {
            println!("Failed to get OpenID configuration {}", err);
            return false;
        }
        println!("ABOBA: {:#?}", self);
        true
    }

    pub async fn get_oidc_discovery_endpoint(&mut self) -> Result<(), OIDCError> {
        let client = reqwest::Client::new();

        let url = format!(
            "https://{}.{}.com/api/discovery/configurations",
            self.customer_id, CLOUD
        );

        let response = client
            .get(url)
            .header("Citrix-ApplicationId", self.application_id.clone())
            .send()
            .await?;

        let status = response.status();

        if status != reqwest::StatusCode::OK {
            return Err(OIDCError::NotOk);
        }

        let json = response.json::<Value>().await?;

        self.acr_values = json["clientSettings"]["acr_values"]
            .as_str()
            .expect("Value should be present and str")
            .to_string();

        self.oidc_discovery_endpoint = json["clientSettings"]["oidcConfiguration"]
            ["oidc_discovery_endpoint"]
            .as_str()
            .expect("Value should be present and str")
            .to_string();

        Ok(())
    }

    pub async fn get_open_id_configuration(&mut self) -> Result<(), OIDCError> {
        let client = reqwest::Client::new();

        let url = format!(
            "https://accounts-internal.{}.com/core/.well-known/openid-configuration",
            CLOUD
        );

        let response = client.get(url).send().await?;

        let status = response.status();

        if status != reqwest::StatusCode::OK {
            return Err(OIDCError::NotOk);
        }

        let json = response.json::<Value>().await?;

        self.authorization_endpoint = json["authorization_endpoint"]
            .as_str()
            .expect("Value should be present and str")
            .to_string();
        self.token_endpoint = json["token_endpoint"]
            .as_str()
            .expect("Value should be present and str")
            .to_string();

        Ok(())
    }
}
