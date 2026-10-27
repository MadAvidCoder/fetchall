use anyhow::{Context, Result};
use reqwest::blocking::Client;
use tiny_http::{Header, Response, Server};

const CLIENT_ID: &str = "JY5N9DSWmLfAS2s8FubiB2vq4AHu2esBk-tUmN18mlM";
const REDIRECT_URI: &str = "http://127.0.0.1:8787/hackatime_callback";

pub fn get_client() -> Result<Client> {
    Ok(Client::builder().user_agent("fetchall").build()?)
}

pub fn login() -> Result<()> {
    let url = format!(
        "https://hackatime.hackclub.com/oauth/authorize?client_id={}&redirect_uri={}&response_type=code&scope=profile+read",
        CLIENT_ID, REDIRECT_URI
    );
    println!("Please go to {} to authorize the application", url);
    open::that(url)?;

    let code = await_callback().context("Callback failed. Please try again later.")?;

    let client = get_client()?;
    let token_response = client
        .post("https://hackatime.hackclub.com/oauth/token")
        .form(&[
            ("client_id", CLIENT_ID),
            ("code", &code),
            ("grant_type", "authorization_code"),
            ("redirect_uri", REDIRECT_URI),
        ])
        .send()
        .context("Failed to send request to Hackatime for token")?;
    if !token_response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to get token from Hackatime. Please try again later. Status: {}",
            token_response.status()
        ));
    }
    let binding = token_response
        .json::<serde_json::Value>()
        .context("Failed to parse JSON response from Hackatime")?;
    let token = binding
        .get("access_token")
        .context("Failed to get access token from JSON response")?
        .as_str()
        .context("Access token is not a string")?;

    let entry = keyring::Entry::new("fetchall", "hackatime-token")
        .context("Failed to create keyring entry")?;
    entry
        .set_password(&token)
        .context("Failed to set keyring password. Check your OS settings")?;
    println!("{}", token);
    print!("Successfully logged in to Hackatime. Access token securely saved to OS store.");
    Ok(())
}

fn await_callback() -> Result<String> {
    let server = Server::http("127.0.0.1:8787").unwrap();
    loop {
        let request = match server.recv() {
            Ok(request) => request,
            Err(e) => {
                eprintln!("Error receiving request: {}", e);
                continue;
            }
        };
        let url = request.url().to_owned();
        if url.starts_with("/hackatime_callback") {
            let queries = url
                .split('?')
                .nth(1)
                .unwrap_or("")
                .split('&')
                .map(|s| {
                    let x = s.split('=');
                    (
                        x.clone().next().unwrap_or(""),
                        x.clone().nth(1).unwrap_or(""),
                    )
                })
                .collect::<Vec<(&str, &str)>>();
            if queries.iter().any(|(k, _)| *k == "code") {
                let code = queries
                    .iter()
                    .find(|(k, _)| *k == "code")
                    .map(|(_, v)| *v)
                    .unwrap_or("");
                request.respond(Response::from_string("<!DOCTYPE html><html><body><script>window.close();</script><p>You can close this tab now :)</p></body></html>").with_header(Header::from_bytes("Content-Type", "text/html; charset=utf-8").unwrap()))?;
                return Ok(code.to_string());
            }
        }
    }
}

pub fn make_request_with_token(
    req: reqwest::blocking::RequestBuilder,
) -> Result<reqwest::blocking::Response> {
    let entry = keyring::Entry::new("fetchall", "hackatime-token")
        .context("Failed to access OS credential")?;
    let token = entry
        .get_password()
        .context("No saved hackatime token. Please run `fetchall auth hackatime` first")?
        .to_string();
    let token = token.trim().trim_matches('"');

    let req = req.header("Authorization", format!("Bearer {}", token));

    req.send()?
        .error_for_status()
        .context("Failed to make request. Your token may have expired.")
}
