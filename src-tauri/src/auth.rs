use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;

const BASE_URL: &str = "https://idcs-8e8265d058d54299bdc845382c75339f.identity.oraclecloud.com";

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CauseMessage {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub status: String,
    #[serde(rename = "ecId")]
    pub ec_id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "nextAuthFactors")]
    pub next_auth_factors: Vec<String>,
    pub cause: Vec<CauseMessage>,
    #[serde(rename = "nextOp")]
    pub next_op: Vec<String>,
    pub scenario: String,
    #[serde(rename = "requestState")]
    pub request_state: String,
    #[serde(rename = "authnToken", skip_serializing_if = "Option::is_none")]
    pub authn_token: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct CredentialsRequest<'a> {
    op: &'a str,
    credentials: Option<Credentials<'a>>,
    #[serde(rename = "requestState")]
    request_state: &'a str,
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct Credentials<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitAuthResponse {
    #[serde(rename = "requestState")]
    pub request_state: String,
}

#[tauri::command]
pub async fn initiate_auth(username: String, password: String) -> Result<AuthResponse, String> {
    // Step 1: Get client credentials token
    println!("Step 1: Getting client credentials token");
    let client_id = env::var("OCI_CLIENT_ID").map_err(|e| e.to_string())?;
    let client_secret = env::var("OCI_CLIENT_SECRET").map_err(|e| e.to_string())?;
    
    let credentials = format!("{}:{}", client_id, client_secret);
    let encoded_credentials = STANDARD.encode(credentials);
    let auth_header = format!("Basic {}", encoded_credentials);
    
    let token_response = get_client_credentials_token(&auth_header)
        .await
        .map_err(|e| {
            println!("Failed to get client credentials token: {}", e);
            e
        })?;
    println!("Successfully obtained access token");

    // Step 2: Initialize authentication
    println!("Step 2: Initializing authentication");
    let bearer_token = format!("Bearer {}", token_response.access_token);
    let init_response = initialize_authentication(&bearer_token)
        .await
        .map_err(|e| {
            println!("Failed to initialize authentication: {}", e);
            e
        })?;
    println!("Successfully initialized authentication");

    // Step 3: Submit credentials
    println!("Step 3: Submitting credentials");
    let client = reqwest::Client::new();
    let cred_url = format!("{}/sso/v1/sdk/authenticate", BASE_URL);
    
    let cred_request = json!({
        "op": "credSubmit",
        "credentials": {
            "username": username,
            "password": password
        },
        "requestState": init_response.request_state
    });

    println!("Making request to URL: {}", cred_url);
    println!("Request body structure: {}", serde_json::json!({
        "op": "credSubmit",
        "credentials": {
            "username": "***",
            "password": "***"
        },
        "requestState": "***"
    }));

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&bearer_token).map_err(|e| e.to_string())?,
    );
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );

    let response = client
        .post(&cred_url)
        .headers(headers)
        .json(&cred_request)
        .send()
        .await
        .map_err(|e| {
            println!("Request failed: {}", e);
            e.to_string()
        })?;

    println!("Response status: {}", response.status());
    println!("Response headers: {:#?}", response.headers());
    
    let status = response.status();
    let response_text = response.text().await.map_err(|e| {
        println!("Failed to get response text: {}", e);
        e.to_string()
    })?;
    println!("Response body: {}", response_text);

    if !status.is_success() {
        return Err(format!(
            "Failed to get response: {}",
            response_text
        ));
    }

    let response_json: AuthResponse = serde_json::from_str(&response_text)
        .map_err(|e| {
            println!("Failed to parse response as JSON: {}", e);
            format!("Failed to parse response: {}. Response text: {}", e, response_text)
        })?;

    println!("Successfully parsed response into AuthResponse");
    Ok(response_json)
}

#[tauri::command]
pub async fn complete_auth(request_state: String) -> Result<Value, String> {
    // Step 1: Get client credentials token
    println!("Step 1: Getting client credentials token");
    let client_id = env::var("OCI_CLIENT_ID").map_err(|e| e.to_string())?;
    let client_secret = env::var("OCI_CLIENT_SECRET").map_err(|e| e.to_string())?;
    let auth_string = format!("{}:{}", client_id, client_secret);
    let auth_header = format!("Basic {}", STANDARD.encode(auth_string));
    
    let token_response = get_client_credentials_token(&auth_header)
        .await
        .map_err(|e| {
            println!("Failed to get client credentials token: {}", e);
            e
        })?;
    println!("Successfully obtained access token");

    // Step 4: Complete authentication
    println!("Step 4: Completing authentication");
    let bearer_token = format!("Bearer {}", token_response.access_token);
    let complete_url = format!("{}/sso/v1/sdk/authenticate", BASE_URL);

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&bearer_token).map_err(|e| e.to_string())?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    println!("Making request to URL: {}", complete_url);
    println!("Request headers: Authorization: Bearer *****, Content-Type: application/json");
    println!("Request body: {}", serde_json::json!({
        "op": "credSubmit",
        "requestState": request_state
    }));

    let response = client
        .post(&complete_url)
        .headers(headers)
        .json(&json!({
            "op": "credSubmit",
            "requestState": request_state
        }))
        .send()
        .await
        .map_err(|e| {
            println!("Failed to complete authentication: {}", e);
            e.to_string()
        })?;

    println!("Response status: {}", response.status());
    println!("Response headers: {:#?}", response.headers());

    if !response.status().is_success() {
        println!("Authentication failed with status: {}", response.status());
        return Err(format!("Authentication failed with status: {}", response.status()));
    }

    let response_text = response.text().await.map_err(|e| {
        println!("Failed to get response text: {}", e);
        e.to_string()
    })?;
    println!("Response body: {}", response_text);

    let response_json: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| {
            println!("Failed to parse response JSON: {}", e);
            format!("Failed to parse response JSON: {}. Response text: {}", e, response_text)
        })?;

    if response_json["status"] != "success" {
        return Err(format!("Authentication failed: {}", response_text));
    }

    // Step 5: Exchange token
    println!("Step 5: Exchanging token for access token");
    let token_response = get_token_with_assertion(&auth_header, &response_json["authnToken"].as_str().unwrap())
        .await
        .map_err(|e| {
            println!("Failed to exchange token: {}", e);
            e
        })?;
    
    // Step 6: Get user profile
    println!("Step 6: Getting user profile");
    let bearer_token = format!("Bearer {}", token_response.access_token);
    let user_profile = get_user_profile(&bearer_token)
        .await
        .map_err(|e| {
            println!("Failed to get user profile: {}", e);
            e
        })?;
        
    println!("Successfully retrieved user profile");
    Ok(user_profile)
}

async fn get_client_credentials_token(auth_header: &str) -> Result<TokenResponse, String> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(auth_header).map_err(|e| e.to_string())?,
    );
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/x-www-form-urlencoded"),
    );

    println!("Making token request to URL: {}/oauth2/v1/token", BASE_URL);
    println!("Request headers: Authorization: Basic *****, Content-Type: application/x-www-form-urlencoded");
    println!("Request form data: grant_type=client_credentials, scope=urn:opc:idm:__myscopes__");

    let response = client
        .post(&format!("{}/oauth2/v1/token", BASE_URL))
        .headers(headers)
        .form(&[
            ("grant_type", "client_credentials"),
            ("scope", "urn:opc:idm:__myscopes__"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    println!("Response status: {}", response.status());
    println!("Response headers: {:#?}", response.headers());
    
    let status = response.status();
    let response_text = response.text().await.map_err(|e| e.to_string())?;
    println!("Response body: {}", response_text);

    if !status.is_success() {
        return Err(format!("Failed to get token: {}", response_text));
    }

    let token_response: TokenResponse = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse token response: {}. Response text: {}", e, response_text))?;

    Ok(token_response)
}

async fn initialize_authentication(bearer_token: &str) -> Result<InitAuthResponse, String> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(bearer_token).map_err(|e| e.to_string())?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    println!("Making auth init request to URL: {}/sso/v1/sdk/authenticate", BASE_URL);
    println!("Request headers: Authorization: Bearer *****, Content-Type: application/json");

    let response = client
        .get(&format!("{}/sso/v1/sdk/authenticate", BASE_URL))
        .headers(headers)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    println!("Response status: {}", response.status());
    println!("Response headers: {:#?}", response.headers());
    
    let status = response.status();
    let response_text = response.text().await.map_err(|e| e.to_string())?;
    println!("Response body: {}", response_text);

    if !status.is_success() {
        return Err(format!("Failed to initialize auth: {}", response_text));
    }

    let init_response: InitAuthResponse = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse init response: {}. Response text: {}", e, response_text))?;

    Ok(init_response)
}

async fn get_token_with_assertion(auth_header: &str, authn_token: &str) -> Result<TokenResponse, String> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(auth_header).map_err(|e| e.to_string())?,
    );
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/x-www-form-urlencoded"),
    );

    println!("Making token exchange request to URL: {}/oauth2/v1/token", BASE_URL);
    println!("Request headers: Authorization: Basic *****, Content-Type: application/x-www-form-urlencoded");
    println!("Request form data: grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer, scope=urn:opc:idm:__myscopes__, assertion=*****");

    let response = client
        .post(&format!("{}/oauth2/v1/token", BASE_URL))
        .headers(headers)
        .form(&[
            (
                "grant_type",
                "urn:ietf:params:oauth:grant-type:jwt-bearer",
            ),
            ("scope", "urn:opc:idm:__myscopes__"),
            ("assertion", authn_token),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    println!("Response status: {}", response.status());
    println!("Response headers: {:#?}", response.headers());
    
    let status = response.status();
    let response_text = response.text().await.map_err(|e| e.to_string())?;
    println!("Response body: {}", response_text);

    if !status.is_success() {
        return Err(format!("Failed to get token: {}", response_text));
    }

    let token_response: TokenResponse = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse token response: {}. Response text: {}", e, response_text))?;

    Ok(token_response)
}

async fn get_user_profile(bearer_token: &str) -> Result<Value, String> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(bearer_token).map_err(|e| e.to_string())?,
    );

    println!("Making user profile request to URL: {}/admin/v1/Me", BASE_URL);
    println!("Request headers: Authorization: Bearer *****, Content-Type: application/json");

    let response = client
        .get(&format!("{}/admin/v1/Me", BASE_URL))
        .headers(headers)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    println!("Response status: {}", response.status());
    println!("Response headers: {:#?}", response.headers());
    
    let status = response.status();
    let response_text = response.text().await.map_err(|e| e.to_string())?;
    println!("Response body: {}", response_text);

    if !status.is_success() {
        return Err(format!("Failed to get user profile: {}", response_text));
    }

    let profile: Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse profile response: {}. Response text: {}", e, response_text))?;

    Ok(profile)
}
