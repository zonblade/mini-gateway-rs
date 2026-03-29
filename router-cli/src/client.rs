use crate::auth;
use crate::error::CliError;
use log::debug;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    success: bool,
    token: Option<String>,
    message: String,
}

pub struct ApiClient {
    http: reqwest::Client,
    base_url: String,
    token: Option<String>,
    /// Credentials for re-auth on 401
    username: Option<String>,
    password: Option<String>,
    no_cache: bool,
}

impl ApiClient {
    pub fn new(base_url: &str, no_cache: bool) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            token: None,
            username: None,
            password: None,
            no_cache,
        }
    }

    /// Set credentials and authenticate. Uses cached token if available.
    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<(), CliError> {
        self.username = Some(username.to_string());
        self.password = Some(password.to_string());

        if !self.no_cache {
            if let Some(cached) = auth::read_cached_token(&self.base_url) {
                debug!("Using cached token");
                self.token = Some(cached);
                return Ok(());
            }
        }

        self.login(username, password).await
    }

    async fn login(&mut self, username: &str, password: &str) -> Result<(), CliError> {
        let url = format!("{}/api/v1/users/login", self.base_url);
        debug!("Authenticating as {username}");

        let resp = self
            .http
            .post(&url)
            .json(&LoginRequest {
                username: username.to_string(),
                password: password.to_string(),
            })
            .send()
            .await?;

        let login: LoginResponse = resp.json().await?;

        if !login.success {
            return Err(CliError::Auth(login.message));
        }

        let token = login
            .token
            .ok_or_else(|| CliError::Auth("no token in login response".into()))?;

        if !self.no_cache {
            let _ = auth::write_cached_token(&self.base_url, &token);
        }

        self.token = Some(token);
        Ok(())
    }

    /// Re-authenticate using stored credentials (called on 401).
    async fn reauth(&mut self) -> Result<(), CliError> {
        let (user, pass) = match (&self.username, &self.password) {
            (Some(u), Some(p)) => (u.clone(), p.clone()),
            _ => {
                return Err(CliError::Auth(
                    "no credentials for re-authentication".into(),
                ))
            }
        };
        auth::clear_cached_token();
        self.login(&user, &pass).await
    }

    fn auth_header(&self) -> Result<String, CliError> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| CliError::Auth("not authenticated".into()))?;
        Ok(format!("Bearer {token}"))
    }

    pub async fn get<T: DeserializeOwned>(&mut self, path: &str) -> Result<T, CliError> {
        let url = format!("{}{path}", self.base_url);
        let auth = self.auth_header()?;

        let resp = self
            .http
            .get(&url)
            .header("Authorization", &auth)
            .send()
            .await?;

        if resp.status().as_u16() == 401 {
            self.reauth().await?;
            let auth = self.auth_header()?;
            let resp = self
                .http
                .get(&url)
                .header("Authorization", &auth)
                .send()
                .await?;
            return Self::handle_json_response(resp).await;
        }

        Self::handle_json_response(resp).await
    }

    pub async fn post_json<B: Serialize, T: DeserializeOwned>(
        &mut self,
        path: &str,
        body: &B,
    ) -> Result<T, CliError> {
        let url = format!("{}{path}", self.base_url);
        let auth = self.auth_header()?;

        let resp = self
            .http
            .post(&url)
            .header("Authorization", &auth)
            .json(body)
            .send()
            .await?;

        if resp.status().as_u16() == 401 {
            self.reauth().await?;
            let auth = self.auth_header()?;
            let resp = self
                .http
                .post(&url)
                .header("Authorization", &auth)
                .json(body)
                .send()
                .await?;
            return Self::handle_json_response(resp).await;
        }

        Self::handle_json_response(resp).await
    }

    pub async fn put_json<B: Serialize, T: DeserializeOwned>(
        &mut self,
        path: &str,
        body: &B,
    ) -> Result<T, CliError> {
        let url = format!("{}{path}", self.base_url);
        let auth = self.auth_header()?;

        let resp = self
            .http
            .put(&url)
            .header("Authorization", &auth)
            .json(body)
            .send()
            .await?;

        if resp.status().as_u16() == 401 {
            self.reauth().await?;
            let auth = self.auth_header()?;
            let resp = self
                .http
                .put(&url)
                .header("Authorization", &auth)
                .json(body)
                .send()
                .await?;
            return Self::handle_json_response(resp).await;
        }

        Self::handle_json_response(resp).await
    }

    pub async fn delete_text(&mut self, path: &str) -> Result<String, CliError> {
        let url = format!("{}{path}", self.base_url);
        let auth = self.auth_header()?;

        let resp = self
            .http
            .delete(&url)
            .header("Authorization", &auth)
            .send()
            .await?;

        if resp.status().as_u16() == 401 {
            self.reauth().await?;
            let auth = self.auth_header()?;
            let resp = self
                .http
                .delete(&url)
                .header("Authorization", &auth)
                .send()
                .await?;
            return Self::handle_text_response(resp).await;
        }

        Self::handle_text_response(resp).await
    }

    pub async fn delete<T: DeserializeOwned>(&mut self, path: &str) -> Result<T, CliError> {
        let url = format!("{}{path}", self.base_url);
        let auth = self.auth_header()?;

        let resp = self
            .http
            .delete(&url)
            .header("Authorization", &auth)
            .send()
            .await?;

        if resp.status().as_u16() == 401 {
            self.reauth().await?;
            let auth = self.auth_header()?;
            let resp = self
                .http
                .delete(&url)
                .header("Authorization", &auth)
                .send()
                .await?;
            return Self::handle_json_response(resp).await;
        }

        Self::handle_json_response(resp).await
    }

    pub async fn post_string<T: DeserializeOwned>(
        &mut self,
        path: &str,
        body: &str,
        content_type: &str,
    ) -> Result<T, CliError> {
        let url = format!("{}{path}", self.base_url);
        let auth = self.auth_header()?;

        let resp = self
            .http
            .post(&url)
            .header("Authorization", &auth)
            .header("Content-Type", content_type)
            .body(body.to_string())
            .send()
            .await?;

        if resp.status().as_u16() == 401 {
            self.reauth().await?;
            let auth = self.auth_header()?;
            let resp = self
                .http
                .post(&url)
                .header("Authorization", &auth)
                .header("Content-Type", content_type)
                .body(body.to_string())
                .send()
                .await?;
            return Self::handle_json_response(resp).await;
        }

        Self::handle_json_response(resp).await
    }

    pub async fn get_string(&mut self, path: &str) -> Result<String, CliError> {
        let url = format!("{}{path}", self.base_url);
        let auth = self.auth_header()?;

        let resp = self
            .http
            .get(&url)
            .header("Authorization", &auth)
            .send()
            .await?;

        if resp.status().as_u16() == 401 {
            self.reauth().await?;
            let auth = self.auth_header()?;
            let resp = self
                .http
                .get(&url)
                .header("Authorization", &auth)
                .send()
                .await?;
            return Self::handle_text_response(resp).await;
        }

        Self::handle_text_response(resp).await
    }

    async fn handle_json_response<T: DeserializeOwned>(
        resp: reqwest::Response,
    ) -> Result<T, CliError> {
        let status = resp.status().as_u16();
        if status >= 400 {
            let message = resp.text().await.unwrap_or_else(|_| "unknown error".into());
            return Err(CliError::Api { status, message });
        }
        Ok(resp.json().await?)
    }

    async fn handle_text_response(resp: reqwest::Response) -> Result<String, CliError> {
        let status = resp.status().as_u16();
        if status >= 400 {
            let message = resp.text().await.unwrap_or_else(|_| "unknown error".into());
            return Err(CliError::Api { status, message });
        }
        Ok(resp.text().await?)
    }
}
