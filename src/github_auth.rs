use anyhow::{Context, Result, bail};
use reqwest::blocking::Client;
use std::time::{Duration, Instant};

const CLIENT_ID: &str = "Iv23liyYTQ6uxsZGC7kg";
const DEVICE_URL: &str = "https://github.com/login/device/code";
const GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";

#[derive(Debug, serde::Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

fn request_device_code() -> Result<DeviceCodeResponse> {
    let client = Client::builder().user_agent("fetchall").build()?;
    let params = [("client_id", CLIENT_ID)];
    let response: DeviceCodeResponse = client
        .post(DEVICE_URL)
        .header("Accept", "application/json")
        .form(&params)
        .send()?
        .error_for_status()?
        .json()
        .context("Failed to get device id")?;
    Ok(response)
}

fn request_access_token(device_code: &str) -> Result<String> {
    let client = Client::builder().user_agent("fetchall").build()?;
    let params = [
        ("client_id", CLIENT_ID),
        ("device_code", device_code),
        ("grant_type", GRANT_TYPE),
    ];
    let response: serde_json::Value = client
        .post(TOKEN_URL)
        .header("Accept", "application/json")
        .form(&params)
        .send()?
        .error_for_status()?
        .json()
        .context("Failed to check for access token")?;

    if let Some(access_token) = response.get("access_token") {
        return Ok(access_token.to_string());
    }
    if let Some(error) = response.get("error") {
        return Err(anyhow::anyhow!(error.to_string()));
    }

    Err(anyhow::anyhow!("Failed to get access token"))
}

pub fn login() -> Result<()> {
    let device = request_device_code()?;
    println!(
        "Please go to {} and enter the code: {}",
        device.verification_uri, device.user_code
    );
    open::that(&device.verification_uri)?;
    let started = Instant::now();
    let mut interval = device.interval.max(1);
    loop {
        if started.elapsed() >= Duration::from_secs(device.expires_in) {
            return Err(anyhow::anyhow!(
                "Device code expired. Please re-run the command"
            ));
        }
        std::thread::sleep(Duration::from_secs(interval));

        match request_access_token(device.device_code.as_str()) {
            Ok(token) => {
                let entry = keyring::Entry::new("fetchall", "github-token")
                    .context("Failed to create keyring entry")?;
                entry
                    .set_password(&token)
                    .context("Failed to set keyring password")?;
                print!(
                    "Successfully logged in to GitHub. Access token securely saved to OS store."
                );
                return Ok(());
            }
            Err(e) => {
                if e.to_string().contains("authorization_pending") {
                    continue;
                } else if e.to_string().contains("slow_down") {
                    interval += 5;
                    continue;
                } else if e.to_string().contains("access_denied") {
                    bail!("Github authorisation denied. Please try again later.");
                } else if e.to_string().contains("expired_token")
                    || e.to_string().contains("token_expired")
                {
                    bail!("Github device code expired. Please rerun the command.");
                } else {
                    return Err(e);
                }
            }
        }
    }
}

pub fn username() -> Result<String> {
    let entry = keyring::Entry::new("fetchall", "github-token")
        .context("Failed to access OS credential store")?;
    let token = entry
        .get_password()
        .context("No saved github token. Please run `fetchall auth github` first")?
        .to_string();
    let token = token.trim().trim_matches('"');

    let client = Client::builder().user_agent("fetchall").build()?;
    let user = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .send()?
        .error_for_status()
        .context("Failed to get GitHub user info. Your token may have expired.")?
        .json::<serde_json::Value>()
        .context("Failed to get GitHub user info")?;

    Ok(user
        .get("login")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Failed to get GitHub username"))?
        .to_string())
}

pub fn get_client() -> Result<Client> {
    Ok(Client::builder().user_agent("fetchall").build()?)
}

pub fn make_request_with_token(
    req: reqwest::blocking::RequestBuilder,
) -> Result<serde_json::Value> {
    let entry = keyring::Entry::new("fetchall", "github-token")
        .context("Failed to access OS credential")?;
    let token = entry
        .get_password()
        .context("No saved github token. Please run `fetchall auth github` first")?
        .to_string();
    let token = token.trim().trim_matches('"');

    let req = req
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json");

    req.send()?
        .error_for_status()
        .context("Failed to make request. Your token may have expired.")?
        .json::<serde_json::Value>()
        .context("Failed to parse response")
}
