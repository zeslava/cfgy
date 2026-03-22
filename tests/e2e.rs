use reqwest::Client;
use serde_json::{json, Value};
use std::sync::OnceLock;
use std::time::Duration;

const BASE_URL: &str = "http://localhost:8080";

struct SharedAuth {
    email: String,
    token: String,
}

static AUTH: OnceLock<SharedAuth> = OnceLock::new();

fn shared_auth() -> &'static SharedAuth {
    AUTH.get().expect("shared auth not initialized — run e2e_setup first")
}

fn auth_headers(token: &str) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Authorization",
        format!("Bearer {token}").parse().unwrap(),
    );
    headers
}

async fn wait_for_server(client: &Client) {
    for _ in 0..60 {
        if let Ok(resp) = client
            .get(format!("{BASE_URL}/health"))
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            if resp.status().is_success() {
                return;
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    panic!("Сервер не поднялся за 60 секунд");
}

async fn setup_auth(client: &Client) -> &'static SharedAuth {
    if let Some(auth) = AUTH.get() {
        return auth;
    }

    let email = format!("e2e-{}@test.local", uuid::Uuid::new_v4());
    let password = "testpass123456";

    let resp = client
        .post(format!("{BASE_URL}/api/auth/register"))
        .json(&json!({"email": email, "password": password}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    let resp = client
        .post(format!("{BASE_URL}/api/auth/login"))
        .json(&json!({"email": email, "password": password}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let token = body["session_token"]
        .as_str()
        .expect("нет session_token")
        .to_string();

    let _ = AUTH.set(SharedAuth {
        email: email.clone(),
        token,
    });
    AUTH.get().unwrap()
}

// --- Health ---

#[tokio::test]
async fn e2e_health() {
    let c = Client::new();
    wait_for_server(&c).await;
    let resp = c.get(format!("{BASE_URL}/health")).send().await.unwrap();
    assert_eq!(resp.status(), 200);
}

// --- Auth ---

#[tokio::test]
async fn e2e_auth_me() {
    let c = Client::new();
    wait_for_server(&c).await;
    let auth = setup_auth(&c).await;

    let resp = c
        .get(format!("{BASE_URL}/api/auth/me"))
        .headers(auth_headers(&auth.token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let me: Value = resp.json().await.unwrap();
    assert_eq!(me["email"].as_str().unwrap(), auth.email);
}

#[tokio::test]
async fn e2e_auth_duplicate_register() {
    let c = Client::new();
    wait_for_server(&c).await;
    let auth = setup_auth(&c).await;

    let resp = c
        .post(format!("{BASE_URL}/api/auth/register"))
        .json(&json!({"email": auth.email, "password": "whatever"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn e2e_auth_wrong_password() {
    let c = Client::new();
    wait_for_server(&c).await;
    let auth = setup_auth(&c).await;

    let resp = c
        .post(format!("{BASE_URL}/api/auth/login"))
        .json(&json!({"email": auth.email, "password": "wrongpassword"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn e2e_auth_unauthorized() {
    let c = Client::new();
    wait_for_server(&c).await;

    let resp = c
        .get(format!("{BASE_URL}/api/projects"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

// --- Projects CRUD ---

#[tokio::test]
async fn e2e_project_crud() {
    let c = Client::new();
    wait_for_server(&c).await;
    let auth = setup_auth(&c).await;
    let h = auth_headers(&auth.token);

    // Create
    let resp = c
        .post(format!("{BASE_URL}/api/projects"))
        .headers(h.clone())
        .json(&json!({"name": "e2e-proj-crud", "description": "crud test"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let project: Value = resp.json().await.unwrap();
    let id = project["id"].as_str().unwrap();

    // List
    let resp = c
        .get(format!("{BASE_URL}/api/projects"))
        .headers(h.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let list: Value = resp.json().await.unwrap();
    assert!(list.as_array().unwrap().iter().any(|p| p["id"] == id));

    // Get
    let resp = c
        .get(format!("{BASE_URL}/api/projects/{id}"))
        .headers(h.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let p: Value = resp.json().await.unwrap();
    assert_eq!(p["name"].as_str().unwrap(), "e2e-proj-crud");

    // Update
    let resp = c
        .patch(format!("{BASE_URL}/api/projects/{id}"))
        .headers(h.clone())
        .json(&json!({"name": "e2e-proj-renamed"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let p: Value = resp.json().await.unwrap();
    assert_eq!(p["name"].as_str().unwrap(), "e2e-proj-renamed");

    // Delete
    let resp = c
        .delete(format!("{BASE_URL}/api/projects/{id}"))
        .headers(h.clone())
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    // Verify deleted
    let resp = c
        .get(format!("{BASE_URL}/api/projects/{id}"))
        .headers(h.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

// --- Configurations ---

#[tokio::test]
async fn e2e_configurations() {
    let c = Client::new();
    wait_for_server(&c).await;
    let auth = setup_auth(&c).await;
    let h = auth_headers(&auth.token);

    // Setup: create project
    let resp = c
        .post(format!("{BASE_URL}/api/projects"))
        .headers(h.clone())
        .json(&json!({"name": "e2e-proj-configs"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let project: Value = resp.json().await.unwrap();
    let pid = project["id"].as_str().unwrap();

    // Create configuration
    let resp = c
        .post(format!("{BASE_URL}/api/projects/{pid}/configurations"))
        .headers(h.clone())
        .json(&json!({"name": "staging"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    // List
    let resp = c
        .get(format!("{BASE_URL}/api/projects/{pid}/configurations"))
        .headers(h.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let configs: Value = resp.json().await.unwrap();
    let cfg = configs
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "staging")
        .expect("staging not found");
    let cfg_id = cfg["id"].as_str().unwrap();

    // Delete configuration
    let resp = c
        .delete(format!(
            "{BASE_URL}/api/projects/{pid}/configurations/{cfg_id}"
        ))
        .headers(h.clone())
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    // Delete project
    let resp = c
        .delete(format!("{BASE_URL}/api/projects/{pid}"))
        .headers(h.clone())
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
}

// --- Parameters ---

#[tokio::test]
async fn e2e_parameters() {
    let c = Client::new();
    wait_for_server(&c).await;
    let auth = setup_auth(&c).await;
    let h = auth_headers(&auth.token);

    // Setup: project + config
    let resp = c
        .post(format!("{BASE_URL}/api/projects"))
        .headers(h.clone())
        .json(&json!({"name": "e2e-proj-params"}))
        .send()
        .await
        .unwrap();
    let project: Value = resp.json().await.unwrap();
    let pid = project["id"].as_str().unwrap();

    c.post(format!("{BASE_URL}/api/projects/{pid}/configurations"))
        .headers(h.clone())
        .json(&json!({"name": "dev"}))
        .send()
        .await
        .unwrap();

    let params_url = format!("{BASE_URL}/api/projects/{pid}/configurations/dev/parameters");

    // Set
    let resp = c
        .put(&params_url)
        .headers(h.clone())
        .json(&json!({"DB_HOST": "db.local", "LOG_LEVEL": "error", "SECRET": "s3cret"}))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    // Get and verify
    let resp = c
        .get(&params_url)
        .headers(h.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let params: Value = resp.json().await.unwrap();
    assert_eq!(params["DB_HOST"].as_str().unwrap(), "db.local");
    assert_eq!(params["LOG_LEVEL"].as_str().unwrap(), "error");
    assert_eq!(params["SECRET"].as_str().unwrap(), "s3cret");

    // Overwrite one
    let resp = c
        .put(&params_url)
        .headers(h.clone())
        .json(&json!({"LOG_LEVEL": "info"}))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    let resp = c
        .get(&params_url)
        .headers(h.clone())
        .send()
        .await
        .unwrap();
    let params: Value = resp.json().await.unwrap();
    assert_eq!(params["LOG_LEVEL"].as_str().unwrap(), "info");

    // Delete one
    let resp = c
        .delete(format!("{params_url}/SECRET"))
        .headers(h.clone())
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    let resp = c
        .get(&params_url)
        .headers(h.clone())
        .send()
        .await
        .unwrap();
    let params: Value = resp.json().await.unwrap();
    assert!(params.get("SECRET").is_none());

    // Cleanup
    c.delete(format!("{BASE_URL}/api/projects/{pid}"))
        .headers(h.clone())
        .send()
        .await
        .unwrap();
}

// --- API Tokens ---

#[tokio::test]
async fn e2e_api_tokens() {
    let c = Client::new();
    wait_for_server(&c).await;
    let auth = setup_auth(&c).await;
    let h = auth_headers(&auth.token);

    // Create token
    let resp = c
        .post(format!("{BASE_URL}/api/tokens"))
        .headers(h.clone())
        .json(&json!({"name": "e2e-token", "expires_in_days": 1}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    let api_token = body["token"].as_str().expect("нет token").to_string();
    let token_id = body["token_id"].as_str().expect("нет token_id").to_string();

    // Use API token
    let resp = c
        .get(format!("{BASE_URL}/api/projects"))
        .header("Authorization", format!("Bearer {api_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // List tokens
    let resp = c
        .get(format!("{BASE_URL}/api/tokens"))
        .headers(h.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let tokens: Value = resp.json().await.unwrap();
    assert!(tokens
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["id"] == token_id.as_str()));

    // Delete token
    let resp = c
        .delete(format!("{BASE_URL}/api/tokens/{token_id}"))
        .headers(h.clone())
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    // Verify revoked
    let resp = c
        .get(format!("{BASE_URL}/api/projects"))
        .header("Authorization", format!("Bearer {api_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}
