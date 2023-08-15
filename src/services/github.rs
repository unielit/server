use crate::{
    errors::AppError,
    models::{repositories::RepositoryOwner, Result},
};
use reqwest::*;
use serde_json::json;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserEmail {
    pub email: String,
    verified: bool,
    primary: bool,
    visibility: Option<String>,
}

#[derive(Deserialize)]
pub struct User {
    pub avatar_url: String,
    pub bio: Option<String>,
    pub blog: Option<String>,
    pub business_plus: Option<bool>,
    pub collaborators: Option<i64>,
    pub company: Option<String>,
    pub created_at: String,
    pub disk_usage: Option<i64>,
    pub email: Option<String>,
    pub events_url: String,
    pub followers: i64,
    pub followers_url: String,
    pub following: i64,
    pub following_url: String,
    pub gists_url: String,
    pub gravatar_id: Option<String>,
    pub hireable: Option<bool>,
    pub html_url: String,
    pub id: i64,
    pub ldap_dn: Option<String>,
    pub location: Option<String>,
    pub login: String,
    pub name: Option<String>,
    pub node_id: String,
    pub organizations_url: String,
    pub owned_private_repos: Option<i64>,
    pub plan: Option<Plan>,
    pub private_gists: Option<i64>,
    pub public_gists: i64,
    pub public_repos: i64,
    pub received_events_url: String,
    pub repos_url: String,
    pub site_admin: bool,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub suspended_at: Option<String>,
    pub total_private_repos: Option<i64>,
    pub twitter_username: Option<String>,
    pub two_factor_authentication: Option<bool>,
    pub r#type: String,
    pub updated_at: String,
    pub url: String,
}

#[derive(Deserialize)]
pub struct Plan {
    pub collaborators: i32,
    pub name: String,
    pub space: i32,
    pub private_repos: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileCommit {
    pub content: Option<Content>,
    pub commit: Commit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    pub name: String,
    pub path: String,
    pub sha: String,
    pub size: i64,
    pub url: String,
    pub html_url: String,
    pub git_url: String,
    pub download_url: String,
    pub r#type: String,
    pub _links: Links,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Links {
    #[serde(rename = "self")]
    pub self_link: String,
    pub git: String,
    pub html: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub node_id: String,
    pub url: String,
    pub html_url: String,
    pub author: CommitAgent,
    pub committer: CommitAgent,
    pub message: String,
    pub tree: Tree,
    pub parents: Vec<Parent>,
    pub verification: Verification,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitAgent {
    pub date: String,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tree {
    pub url: String,
    pub sha: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parent {
    pub url: String,
    pub html_url: String,
    pub sha: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Verification {
    pub verified: bool,
    pub reason: String,
    pub signature: Option<String>,
    pub payload: Option<String>,
}

pub struct GitHubAPI {
    client: Client,
    base_url: Url,
}

impl GitHubAPI {
    pub fn new() -> Result<Self> {
        let base_url = Url::parse("https://api.github.com")?;
        let mut default_headers: header::HeaderMap = header::HeaderMap::new();
        default_headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/vnd.github+json"),
        );
        default_headers.insert(
            header::HeaderName::from_static("x-github-api-version"),
            header::HeaderValue::from_static("2022-11-28"),
        );
        default_headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("UnielitServer"),
        );

        let client = reqwest::Client::builder()
            .default_headers(default_headers)
            .build()?;

        Ok(GitHubAPI { client, base_url })
    }

    pub async fn get_user_primary_email(&self, token: &str) -> Result<UserEmail> {
        let token_value = header::HeaderValue::from_str(&["Bearer ", token].concat())?;
        let mut url = self.base_url.clone();
        url.set_path("/user/emails");

        let response = self
            .client
            .get(url)
            .header(header::AUTHORIZATION, token_value)
            .send()
            .await
            .map_err(|e| AppError::GithubAPIError(e.to_string()))?;

        let emails = response
            .json::<Vec<UserEmail>>()
            .await
            .map_err(|e| AppError::GithubAPIError(e.to_string()))?;

        let primary_email = emails.iter().find(|e| e.primary).cloned();

        if let Some(email) = primary_email {
            return Ok(email);
        }

        Err(AppError::GithubAuthError(
            "Didn't find user's primary email!".to_string(),
        ))
    }

    pub async fn get_auth_user(&self, token: &str) -> Result<User> {
        let token_value = header::HeaderValue::from_str(&["Bearer ", token].concat())?;
        let mut url = self.base_url.clone();
        url.set_path("/user");

        let response = self
            .client
            .get(url)
            .header(header::AUTHORIZATION, token_value)
            .send()
            .await
            .map_err(|e| AppError::GithubAPIError(e.to_string()))?;

        response
            .json::<User>()
            .await
            .map_err(|e| AppError::GithubAPIError(e.to_string()))
    }

    pub async fn create_org_repo(
        &self,
        token: &str,
        org: &str,
        body: serde_json::Value,
    ) -> Result<Response> {
        let token_value = header::HeaderValue::from_str(&["Bearer ", token].concat())?;
        let mut url = self.base_url.clone();
        url.set_path(&format!("/orgs/{org}/repos"));

        self.client
            .post(url)
            .header(header::AUTHORIZATION, token_value)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::GithubAPIError(e.to_string()))
    }

    pub async fn create_personal_repo(
        &self,
        token: &str,
        body: serde_json::Value,
    ) -> Result<Response> {
        let token_value = header::HeaderValue::from_str(&["Bearer ", token].concat())?;
        let mut url = self.base_url.clone();
        url.set_path("/user/repos");

        self.client
            .post(url)
            .header(header::AUTHORIZATION, token_value)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::GithubAPIError(e.to_string()))
    }

    // SHA is reequired if you are updating a file. The blob SHA of the file being replaced.
    pub async fn save_file_content(
        &self,
        token: &str,
        repo_owner: RepositoryOwner,
        path: &str,
        message: &str,
        base64_content: &str,
        sha: Option<&str>,
    ) -> Result<FileCommit> {
        let token_value = header::HeaderValue::from_str(&format!("Bearer {token}"))?;
        let mut url = self.base_url.to_owned();
        url.set_path(&format!(
            "/repos/{}/{}/contents/{path}",
            repo_owner.owner, repo_owner.name
        ));

        let json = json!({
            "message" : message,
            "content" : base64_content,
            "sha" : sha
        });

        let response = self
            .client
            .put(url)
            .header(header::AUTHORIZATION, token_value)
            .json(&json)
            .send()
            .await
            .map_err(|e| AppError::GithubAPIError(e.to_string()))?;

        if response.status().is_success() {
            return response
                .json::<FileCommit>()
                .await
                .map_err(|e| AppError::GithubAPIError(e.to_string()));
        }

        Err(AppError::GithubAPIError(response.text().await?))
    }
}
