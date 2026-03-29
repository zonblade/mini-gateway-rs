use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// Re-export the client from the main crate
// Integration tests use the binary's public API through the client module
pub struct TestApiClient {
    http: reqwest::Client,
    base_url: String,
    token: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

impl TestApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            token: None,
            username: None,
            password: None,
        }
    }

    pub fn token(&self) -> Option<String> {
        self.token.clone()
    }

    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<(), String> {
        self.username = Some(username.to_string());
        self.password = Some(password.to_string());

        let url = format!("{}/api/v1/users/login", self.base_url);
        let resp = self
            .http
            .post(&url)
            .json(&serde_json::json!({
                "username": username,
                "password": password,
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

        if body["success"].as_bool() != Some(true) {
            return Err(body["message"]
                .as_str()
                .unwrap_or("Login failed")
                .to_string());
        }

        self.token = body["token"].as_str().map(String::from);
        Ok(())
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token.as_deref().unwrap_or(""))
    }

    async fn reauth(&mut self) -> Result<(), String> {
        let (u, p) = match (&self.username, &self.password) {
            (Some(u), Some(p)) => (u.clone(), p.clone()),
            _ => return Err("No credentials".into()),
        };
        self.authenticate(&u, &p).await
    }

    pub async fn get<T: serde::de::DeserializeOwned>(
        &mut self,
        api_path: &str,
    ) -> Result<T, String> {
        let url = format!("{}{api_path}", self.base_url);
        let resp = self
            .http
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status().as_u16() == 401 {
            self.reauth().await?;
            let resp = self
                .http
                .get(&url)
                .header("Authorization", self.auth_header())
                .send()
                .await
                .map_err(|e| e.to_string())?;
            return handle_response(resp).await;
        }
        handle_response(resp).await
    }

    pub async fn post_json<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &mut self,
        api_path: &str,
        body: &B,
    ) -> Result<T, String> {
        let url = format!("{}{api_path}", self.base_url);
        let resp = self
            .http
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status().as_u16() == 401 {
            self.reauth().await?;
            let resp = self
                .http
                .post(&url)
                .header("Authorization", self.auth_header())
                .json(body)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            return handle_response(resp).await;
        }
        handle_response(resp).await
    }

    pub async fn delete_text(&mut self, api_path: &str) -> Result<String, String> {
        let url = format!("{}{api_path}", self.base_url);
        let resp = self
            .http
            .delete(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.status().as_u16() == 401 {
            self.reauth().await?;
            let resp = self
                .http
                .delete(&url)
                .header("Authorization", self.auth_header())
                .send()
                .await
                .map_err(|e| e.to_string())?;
            return handle_text(resp).await;
        }
        handle_text(resp).await
    }
}

async fn handle_response<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, String> {
    let status = resp.status().as_u16();
    if status >= 400 {
        let msg = resp.text().await.unwrap_or_default();
        return Err(format!("API error ({status}): {msg}"));
    }
    resp.json().await.map_err(|e| e.to_string())
}

async fn handle_text(resp: reqwest::Response) -> Result<String, String> {
    let status = resp.status().as_u16();
    if status >= 400 {
        let msg = resp.text().await.unwrap_or_default();
        return Err(format!("API error ({status}): {msg}"));
    }
    resp.text().await.map_err(|e| e.to_string())
}

/// Mount the login endpoint on a mock server.
pub async fn mount_login(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/api/v1/users/login"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "token": "mock-token",
            "user_id": "test-user-id",
            "username": "admin",
            "role": "admin",
            "message": "Login successful"
        })))
        .mount(server)
        .await;
}

/// Create a client pointing at the mock server.
pub fn make_client(server: &MockServer) -> TestApiClient {
    TestApiClient::new(&server.uri())
}

/// Create an already-authenticated client.
pub async fn authed_client(server: &MockServer) -> TestApiClient {
    let mut client = make_client(server);
    client.authenticate("admin", "adminpassword").await.unwrap();
    client
}
