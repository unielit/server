use crate::{errors::AppError, models::Result};
use reqwest::*;

pub struct GitHubAPI {
    client: Client,
    base_url: Url,
}

impl GitHubAPI {
    pub fn new(token: &str) -> Result<Self> {
        let base_url = Url::parse("https://api.github.com/")?;
        let token_value = header::HeaderValue::from_str(token)?;
        let mut default_headers: header::HeaderMap = header::HeaderMap::new();
        default_headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/vnd.github+json"),
        );
        default_headers.insert(
            header::HeaderName::from_static("x-github-api-version"),
            header::HeaderValue::from_static("2022-11-28"),
        );
        default_headers.insert(header::AUTHORIZATION, token_value);
        default_headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("UnielitServer"),
        );

        let client = reqwest::Client::builder()
            .default_headers(default_headers)
            .build()?;

        Ok(GitHubAPI { client, base_url })
    }

    pub async fn create_org_repo(&self, org: &str, body: serde_json::Value) -> Result<Response> {
        let mut url = self.base_url.clone();
        url.set_path(&format!("/orgs/{org}/repos"));

        self.client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(AppError::from)
    }

    pub async fn create_personal_repo(&self, body: serde_json::Value) -> Result<Response> {
        let mut url = self.base_url.clone();
        url.set_path("/user/repos");

        self.client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(AppError::from)
    }
}
