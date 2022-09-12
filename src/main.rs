use reqwest::{
    self,
    header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE},
};
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, fs, path::Path, thread, time};
use tokio;

#[derive(Deserialize)]
struct Configuration {
    zone_id: Option<String>,
    bearer_token: Option<String>,
    record_id: Option<String>,
    webhook_enabled: Option<bool>,
    webhook_url: Option<String>,
}

#[tokio::main]
async fn main() {
    if !Path::new("config.toml").exists() {
        println!("config.toml does not exist. Please create it.");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("fail");
    }
    let config_string = fs::read_to_string("config.toml").expect("Unable to open file.");
    let config: Configuration = match toml::from_str(&config_string) {
        Ok(conf) => conf,
        Err(err) => panic!("Error occured in parsing toml file: {}", err),
    };
    main_loop(config).await;
}

async fn main_loop(config: Configuration) {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        format!("Bearer {}", config.bearer_token.unwrap())
            .parse()
            .unwrap(),
    );
    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

    let zone_id = config.zone_id.unwrap();
    let record_id = config.record_id.unwrap();
    let webhook_enabled = config.webhook_enabled.unwrap();
    let webhook_url = config.webhook_url.unwrap();
    let mut current_ip = "";

    loop {
        let res = client
            .get(format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                zone_id, record_id
            ))
            .headers(headers.clone())
            .send()
            .await
            .expect("Error occured while parsing first request");

        let res_json = res
            .text()
            .await
            .expect("Error occured while parsing JSON for first request");

        let root: Value = match serde_json::from_str(&res_json) {
            Ok(val) => val,
            Err(err) => {
                println!(
                    "Error occured while parsing JSON for first request: {}",
                    err
                );
                continue;
            }
        };

        let current_set_ip: Option<&str> = root
            .get("result")
            .and_then(|value| value.get("content"))
            .and_then(|value| value.as_str());

        current_ip = current_set_ip.unwrap();

        let mut ip = loop {
            match get_ip().await {
                Ok(ip) => break ip,
                Err(err) => {
                    println!("Error occured in get_ip: {}", err);
                    thread::sleep(time::Duration::from_millis(100));
                    continue;
                }
            }
        };

        while current_ip == ip {
            thread::sleep(time::Duration::from_secs(300));
            ip = loop {
                match get_ip().await {
                    Ok(ip) => break ip,
                    Err(err) => {
                        println!("Error occured in get_ip: {}", err);
                        thread::sleep(time::Duration::from_millis(100));
                        continue;
                    }
                }
            }
        }

        let mut payload = HashMap::new();
        payload.insert("content", &ip);
        let _ = client
            .patch(format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                zone_id, record_id
            ))
            .headers(headers.clone())
            .json(&payload)
            .send()
            .await
            .expect("Sending new IP failed.");

        current_ip = &ip;

        if webhook_enabled {
            let mut discord_message = HashMap::new();
            discord_message.insert("content", ("Changed IP from {} to {}", current_set_ip, &ip));
            let _ = client
                .patch(&webhook_url)
                .json(&payload)
                .send()
                .await
                .expect("Sending discord webhook failed.");
        }
    }
}

async fn get_ip() -> Result<String, reqwest::Error> {
    let mut resp = reqwest::get("https://api.ipify.org/").await?;
    while !resp.status().is_success() {
        thread::sleep(time::Duration::from_secs(10));

        resp = reqwest::get("https://api.ipify.org/").await?;
    }
    let resp_text = resp.text().await?;
    Ok(resp_text)
}
