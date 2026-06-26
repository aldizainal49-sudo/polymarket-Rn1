use serde::Deserialize;
use reqwest::Client;
use anyhow::Result;

#[derive(Debug, Deserialize)]
pub struct SportmonksFixture {
    pub id: u32,
    pub name: String,
    pub starting_at: String,
    pub result_info: Option<String>,
    pub status: String,
    pub participants: Vec<SportmonksParticipant>,
}

#[derive(Debug, Deserialize)]
pub struct SportmonksParticipant {
    pub id: u32,
    pub name: String,
}

pub struct SportmonksClient {
    client: Client,
    api_token: String,
    base_url: String,
}

impl SportmonksClient {
    pub fn new(api_token: String) -> Self {
        Self { client: Client::new(), api_token, base_url: "https://api.sportmonks.com/v3".to_string() }
    }

    pub async fn get_live_matches(&self) -> Result<Vec<SportmonksFixture>> {
        let url = format!("{}/football/fixtures/live?api_token={}&include=participants;scores", self.base_url, self.api_token);
        let response = self.client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;
        if let Some(fixtures) = data.get("data").and_then(|d| d.as_array()) {
            let parsed: Vec<SportmonksFixture> = serde_json::from_value(serde_json::Value::Array(fixtures.clone()))?;
            return Ok(parsed);
        }
        Ok(Vec::new())
    }
}
