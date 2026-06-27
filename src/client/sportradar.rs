use reqwest::Client;
use serde::Deserialize;
use anyhow::Result;

#[derive(Debug, Deserialize)]
pub struct ScheduleResponse {
    pub games: Vec<Game>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Game {
    pub id: String,
    pub status: String,
    pub scheduled: String,
    pub home: Team,
    pub away: Team,
    pub home_score: Option<i32>,
    pub away_score: Option<i32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Debug)]
pub struct SportradarClient {
    client: Client,
    api_key: String,
}

impl SportradarClient {
    pub fn new(api_key: String) -> Self {
        Self { client: Client::new(), api_key }
    }

    fn build_url(&self, sport: &str, endpoint: &str) -> String {
        let base = match sport {
            "soccer" => "https://api.sportradar.com/soccer/trial/v4/en",
            "nba" => "https://api.sportradar.com/nba/trial/v7/en",
            "mlb" => "https://api.sportradar.com/mlb/trial/v7/en",
            "global_basketball" => "https://api.sportradar.com/global-basketball/trial/v7/en",
            _ => "https://api.sportradar.com/soccer/trial/v4/en",
        };
        format!("{}{}?api_key={}", base, endpoint, self.api_key)
    }

    pub async fn get_live_games(&self, sport: &str) -> Result<Vec<Game>> {
        let url = self.build_url(sport, "/games/live");
        let response = self.client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;
        if let Some(games) = data.get("games").and_then(|g| g.as_array()) {
            let parsed: Vec<Game> = serde_json::from_value(serde_json::Value::Array(games.clone()))?;
            return Ok(parsed);
        }
        Ok(Vec::new())
    }
}