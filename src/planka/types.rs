use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub position: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct List {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub position: Option<f64>,
    pub board_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub list_id: String,
    #[serde(default)]
    pub position: Option<f64>,
    #[serde(default)]
    pub board_id: Option<String>,
}

/// Response from GET /api/projects
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectsResponse {
    pub items: Vec<Project>,
}

/// Response from GET /api/projects/{id} (includes nested boards, lists, cards)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResponse {
    #[allow(dead_code)]
    pub item: Project,
    pub included: ProjectIncluded,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIncluded {
    #[serde(default)]
    pub boards: Vec<Board>,
    #[serde(default)]
    #[allow(dead_code)]
    pub lists: Vec<List>,
    #[serde(default)]
    #[allow(dead_code)]
    pub cards: Vec<Card>,
}

/// Response from GET /api/boards/{id}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardResponse {
    #[allow(dead_code)]
    pub item: Board,
    pub included: BoardIncluded,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardIncluded {
    #[serde(default)]
    pub lists: Vec<List>,
    #[serde(default)]
    pub cards: Vec<Card>,
}

/// Response from POST /api/lists/{listId}/cards
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardResponse {
    pub item: Card,
}

/// Request body for creating a card
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCardRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub position: f64,
}

/// Response from POST /api/projects/{projectId}/boards
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardCreateResponse {
    pub item: Board,
}

/// Request body for creating a board
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBoardRequest {
    pub name: String,
    pub position: f64,
}

/// Response from POST /api/boards/{boardId}/lists
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse {
    pub item: List,
}

/// Request body for creating a list
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateListRequest {
    pub name: String,
    pub position: f64,
}

/// Request body for creating a project
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectRequest {
    pub name: String,
}

/// Response from POST /api/projects
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCreateResponse {
    pub item: Project,
}
