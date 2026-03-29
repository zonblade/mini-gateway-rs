use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod helpers;
use helpers::*;

// -- Authentication --

#[tokio::test]
async fn login_success() {
    let server = MockServer::start().await;
    mount_login(&server).await;

    let mut client = make_client(&server);
    client.authenticate("admin", "adminpassword").await.unwrap();
    assert!(client.token().is_some());
}

#[tokio::test]
async fn login_failure() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/users/login"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": false,
            "token": null,
            "message": "Invalid credentials"
        })))
        .mount(&server)
        .await;

    let mut client = make_client(&server);
    let result = client.authenticate("wrong", "wrong").await;
    assert!(result.is_err());
}

// -- Proxy CRUD --

#[tokio::test]
async fn proxy_list_empty() {
    let server = MockServer::start().await;
    mount_login(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/settings/proxies"))
        .and(header("Authorization", "Bearer mock-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    let mut client = authed_client(&server).await;
    let proxies: Vec<serde_json::Value> = client.get("/api/v1/settings/proxies").await.unwrap();
    assert!(proxies.is_empty());
}

#[tokio::test]
async fn proxy_list_with_data() {
    let server = MockServer::start().await;
    mount_login(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/settings/proxies"))
        .and(header("Authorization", "Bearer mock-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "proxy": {
                    "id": "abc-123",
                    "title": "test-proxy",
                    "addr_listen": "0.0.0.0:8080",
                    "addr_target": "127.0.0.1:3000",
                    "high_speed": false,
                    "high_speed_addr": null,
                    "high_speed_gwid": null
                },
                "domains": []
            }
        ])))
        .mount(&server)
        .await;

    let mut client = authed_client(&server).await;
    let proxies: Vec<serde_json::Value> = client.get("/api/v1/settings/proxies").await.unwrap();
    assert_eq!(proxies.len(), 1);
    assert_eq!(proxies[0]["proxy"]["title"], "test-proxy");
}

#[tokio::test]
async fn proxy_create() {
    let server = MockServer::start().await;
    mount_login(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/settings/proxy"))
        .and(header("Authorization", "Bearer mock-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "proxy": {
                "id": "new-id-123",
                "title": "created-proxy",
                "addr_listen": "0.0.0.0:443",
                "addr_target": "",
                "high_speed": false,
                "high_speed_addr": null,
                "high_speed_gwid": null
            },
            "domains": []
        })))
        .mount(&server)
        .await;

    let mut client = authed_client(&server).await;
    let input = serde_json::json!({
        "proxy": {
            "id": "",
            "title": "created-proxy",
            "addr_listen": "0.0.0.0:443",
            "addr_target": "",
            "high_speed": false,
            "high_speed_addr": null,
            "high_speed_gwid": null
        },
        "domains": []
    });
    let result: serde_json::Value = client
        .post_json("/api/v1/settings/proxy", &input)
        .await
        .unwrap();
    assert_eq!(result["proxy"]["id"], "new-id-123");
}

#[tokio::test]
async fn proxy_delete() {
    let server = MockServer::start().await;
    mount_login(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/settings/proxy/abc-123"))
        .and(header("Authorization", "Bearer mock-token"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string("Proxy 'test' deleted. 0 domains removed."),
        )
        .mount(&server)
        .await;

    let mut client = authed_client(&server).await;
    let msg: String = client
        .delete_text("/api/v1/settings/proxy/abc-123")
        .await
        .unwrap();
    assert!(msg.contains("deleted"));
}

// -- 401 Re-authentication --

#[tokio::test]
async fn reauth_on_401() {
    let server = MockServer::start().await;
    mount_login(&server).await;

    // First call returns 401, second succeeds
    Mock::given(method("GET"))
        .and(path("/api/v1/settings/proxies"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": "Unauthorized"
        })))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/v1/settings/proxies"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    let mut client = authed_client(&server).await;
    let proxies: Vec<serde_json::Value> = client.get("/api/v1/settings/proxies").await.unwrap();
    assert!(proxies.is_empty());
}

// -- Error Handling --

#[tokio::test]
async fn api_error_404() {
    let server = MockServer::start().await;
    mount_login(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/settings/proxy/nonexistent"))
        .and(header("Authorization", "Bearer mock-token"))
        .respond_with(
            ResponseTemplate::new(404).set_body_json(serde_json::json!({"error": "Not found"})),
        )
        .mount(&server)
        .await;

    let mut client = authed_client(&server).await;
    let result: Result<serde_json::Value, _> =
        client.get("/api/v1/settings/proxy/nonexistent").await;
    assert!(result.is_err());
}

// -- Gateway Node CRUD --

#[tokio::test]
async fn gwnode_list() {
    let server = MockServer::start().await;
    mount_login(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/settings/gwnode/list"))
        .and(header("Authorization", "Bearer mock-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": "node-1",
                "proxy_id": "proxy-1",
                "title": "backend",
                "alt_target": "127.0.0.1:3000",
                "priority": 100,
                "domain_id": null,
                "domain_name": null
            }
        ])))
        .mount(&server)
        .await;

    let mut client = authed_client(&server).await;
    let nodes: Vec<serde_json::Value> = client.get("/api/v1/settings/gwnode/list").await.unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["title"], "backend");
}

// -- User Management --

#[tokio::test]
async fn user_list() {
    let server = MockServer::start().await;
    mount_login(&server).await;

    Mock::given(method("GET"))
        .and(path("/api/v1/users/admin"))
        .and(header("Authorization", "Bearer mock-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "id": "user-1",
                "username": "admin",
                "email": "admin@test.com",
                "role": "admin",
                "created_at": "2026-01-01",
                "updated_at": null
            }
        ])))
        .mount(&server)
        .await;

    let mut client = authed_client(&server).await;
    let users: Vec<serde_json::Value> = client.get("/api/v1/users/admin").await.unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0]["username"], "admin");
}

// -- Sync --

#[tokio::test]
async fn sync_proxy() {
    let server = MockServer::start().await;
    mount_login(&server).await;

    Mock::given(method("POST"))
        .and(path("/api/v1/sync/proxy"))
        .and(header("Authorization", "Bearer mock-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "success",
            "message": "Proxy data updated"
        })))
        .mount(&server)
        .await;

    let mut client = authed_client(&server).await;
    let result: serde_json::Value = client
        .post_json("/api/v1/sync/proxy", &serde_json::Value::Null)
        .await
        .unwrap();
    assert_eq!(result["status"], "success");
}
