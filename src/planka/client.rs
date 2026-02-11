use reqwest::{Client, header};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use url::Url;

use super::types::*;

#[derive(Debug, Error)]
pub enum PlankaError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("HTTP status {0}: {1}")]
    Status(u16, String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("JSON error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
enum PlankaAuth {
    Token(String),
    Credentials { email: String, password: String },
}

#[derive(Debug)]
pub struct PlankaClient {
    base_url: Url,
    http: Client,
    auth: PlankaAuth,
    cached_token: Arc<RwLock<Option<String>>>,
}

impl PlankaClient {
    pub fn from_env() -> Result<Self, PlankaError> {
        let base_url = std::env::var("PLANKA_URL")
            .map_err(|_| PlankaError::Config("PLANKA_URL not set".into()))?;

        let base_url = Url::parse(&base_url)
            .map_err(|e| PlankaError::Config(format!("Invalid PLANKA_URL: {e}")))?;

        let auth = if let Ok(token) = std::env::var("PLANKA_TOKEN") {
            PlankaAuth::Token(token)
        } else {
            let email = std::env::var("PLANKA_EMAIL")
                .map_err(|_| PlankaError::Config("PLANKA_TOKEN or PLANKA_EMAIL must be set".into()))?;
            let password = std::env::var("PLANKA_PASSWORD")
                .map_err(|_| PlankaError::Config("PLANKA_PASSWORD must be set when using PLANKA_EMAIL".into()))?;
            PlankaAuth::Credentials { email, password }
        };

        let http = Client::builder()
            .build()
            .map_err(PlankaError::Http)?;

        Ok(Self {
            base_url,
            http,
            auth,
            cached_token: Arc::new(RwLock::new(None)),
        })
    }

    async fn get_token(&self) -> Result<String, PlankaError> {
        match &self.auth {
            PlankaAuth::Token(token) => Ok(token.clone()),
            PlankaAuth::Credentials { email, password } => {
                // Check cache first
                {
                    let cache = self.cached_token.read().await;
                    if let Some(token) = cache.as_ref() {
                        return Ok(token.clone());
                    }
                }

                // Fetch new token
                let url = self.base_url.join("/api/access-tokens")?;

                let resp = self.http
                    .post(url)
                    .json(&serde_json::json!({
                        "emailOrUsername": email,
                        "password": password
                    }))
                    .send()
                    .await?;

                if !resp.status().is_success() {
                    let status = resp.status().as_u16();
                    let body = resp.text().await.unwrap_or_default();
                    return Err(PlankaError::Status(status, body));
                }

                let data: serde_json::Value = resp.json().await?;
                let token = data["item"]
                    .as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| PlankaError::Config("No token in login response".into()))?;

                // Cache the token
                {
                    let mut cache = self.cached_token.write().await;
                    *cache = Some(token.clone());
                }

                Ok(token)
            }
        }
    }

    async fn request(&self, method: reqwest::Method, path: &str) -> Result<reqwest::RequestBuilder, PlankaError> {
        let token = self.get_token().await?;
        let url = self.base_url.join(path)?;

        Ok(self.http
            .request(method, url)
            .header(header::AUTHORIZATION, format!("Bearer {token}")))
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>, PlankaError> {
        let resp = self.request(reqwest::Method::GET, "/api/projects")
            .await?
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(PlankaError::Status(status, body));
        }

        let data: ProjectsResponse = resp.json().await?;
        Ok(data.items)
    }

    pub async fn list_boards(&self, project_id: &str) -> Result<Vec<Board>, PlankaError> {
        let path = format!("/api/projects/{project_id}");
        let resp = self.request(reqwest::Method::GET, &path)
            .await?
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(PlankaError::Status(status, body));
        }

        let data: ProjectResponse = resp.json().await?;
        Ok(data.included.boards)
    }

    pub async fn list_cards(&self, board_id: &str) -> Result<Vec<Card>, PlankaError> {
        let path = format!("/api/boards/{board_id}");
        let resp = self.request(reqwest::Method::GET, &path)
            .await?
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(PlankaError::Status(status, body));
        }

        let data: BoardResponse = resp.json().await?;
        Ok(data.included.cards)
    }

    pub async fn list_lists(&self, board_id: &str) -> Result<Vec<List>, PlankaError> {
        let path = format!("/api/boards/{board_id}");
        let resp = self.request(reqwest::Method::GET, &path)
            .await?
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(PlankaError::Status(status, body));
        }

        let data: BoardResponse = resp.json().await?;
        Ok(data.included.lists)
    }

    pub async fn create_project(
        &self,
        name: &str,
    ) -> Result<Project, PlankaError> {
        let body = CreateProjectRequest {
            name: name.to_string(),
        };

        let resp = self.request(reqwest::Method::POST, "/api/projects")
            .await?
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(PlankaError::Status(status, body));
        }

        let data: ProjectCreateResponse = resp.json().await?;
        Ok(data.item)
    }

    pub async fn create_card(
        &self,
        list_id: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<Card, PlankaError> {
        let path = format!("/api/lists/{list_id}/cards");

        let body = CreateCardRequest {
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            position: 65535.0, // Default position at end
        };

        let resp = self.request(reqwest::Method::POST, &path)
            .await?
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(PlankaError::Status(status, body));
        }

        let data: CardResponse = resp.json().await?;
        Ok(data.item)
    }

    pub async fn create_board(
        &self,
        project_id: &str,
        name: &str,
    ) -> Result<Board, PlankaError> {
        let path = format!("/api/projects/{project_id}/boards");

        let body = CreateBoardRequest {
            name: name.to_string(),
            position: 65535.0,
        };

        let resp = self.request(reqwest::Method::POST, &path)
            .await?
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(PlankaError::Status(status, body));
        }

        let data: BoardCreateResponse = resp.json().await?;
        Ok(data.item)
    }

    pub async fn create_list(
        &self,
        board_id: &str,
        name: &str,
    ) -> Result<List, PlankaError> {
        let path = format!("/api/boards/{board_id}/lists");

        let body = CreateListRequest {
            name: name.to_string(),
            position: 65535.0,
        };

        let resp = self.request(reqwest::Method::POST, &path)
            .await?
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(PlankaError::Status(status, body));
        }

        let data: ListResponse = resp.json().await?;
        Ok(data.item)
    }

    pub async fn update_card(
        &self,
        card_id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Card, PlankaError> {
        let path = format!("/api/cards/{card_id}");

        let mut body = serde_json::Map::new();
        if let Some(n) = name {
            body.insert("name".to_string(), serde_json::Value::String(n.to_string()));
        }
        if let Some(d) = description {
            body.insert("description".to_string(), serde_json::Value::String(d.to_string()));
        }

        let resp = self.request(reqwest::Method::PATCH, &path)
            .await?
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(PlankaError::Status(status, body));
        }

        let data: CardResponse = resp.json().await?;
        Ok(data.item)
    }

    pub async fn move_card(
        &self,
        card_id: &str,
        list_id: &str,
        position: Option<f64>,
    ) -> Result<Card, PlankaError> {
        let path = format!("/api/cards/{card_id}");

        let mut body = serde_json::Map::new();
        body.insert("listId".to_string(), serde_json::Value::String(list_id.to_string()));
        let pos = position.unwrap_or(65535.0);
        body.insert("position".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(pos).unwrap()));

        let resp = self.request(reqwest::Method::PATCH, &path)
            .await?
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(PlankaError::Status(status, body));
        }

        let data: CardResponse = resp.json().await?;
        Ok(data.item)
    }

    pub async fn delete_card(&self, card_id: &str) -> Result<(), PlankaError> {
        let path = format!("/api/cards/{card_id}");

        let resp = self.request(reqwest::Method::DELETE, &path)
            .await?
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(PlankaError::Status(status, body));
        }

        Ok(())
    }

    pub async fn delete_list(&self, list_id: &str) -> Result<(), PlankaError> {
        let path = format!("/api/lists/{list_id}");

        let resp = self.request(reqwest::Method::DELETE, &path)
            .await?
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(PlankaError::Status(status, body));
        }

        Ok(())
    }
}

impl From<url::ParseError> for PlankaError {
    fn from(e: url::ParseError) -> Self {
        PlankaError::Config(format!("URL parse error: {e}"))
    }
}
