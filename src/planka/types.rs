use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub position: Option<f64>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct List {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub position: Option<f64>,
    pub board_id: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    #[serde(rename = "type")]
    pub list_type: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
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
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub members: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub labels: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub attachments: Option<Vec<serde_json::Value>>,
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
    #[serde(default)]
    pub board_memberships: Vec<serde_json::Value>,
    #[serde(default)]
    pub labels: Vec<serde_json::Value>,
    #[serde(default)]
    pub users: Vec<serde_json::Value>,
    #[serde(default)]
    pub card_memberships: Vec<serde_json::Value>,
}

/// Response from POST /api/lists/{listId}/cards
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardResponse {
    pub item: Card,
}

/// Response from GET /api/cards/{id}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardDetailResponse {
    pub item: Card,
    pub included: CardIncluded,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardIncluded {
    #[serde(default)]
    pub tasks: Vec<Task>,
    #[serde(default)]
    pub task_lists: Vec<serde_json::Value>,
    #[serde(default)]
    pub card_labels: Vec<serde_json::Value>,
    #[serde(default)]
    pub card_memberships: Vec<serde_json::Value>,
    #[serde(default)]
    pub attachments: Vec<serde_json::Value>,
    #[serde(default)]
    pub users: Vec<serde_json::Value>,
    #[serde(default)]
    pub custom_field_groups: Vec<serde_json::Value>,
    #[serde(default)]
    pub custom_fields: Vec<serde_json::Value>,
    #[serde(default)]
    pub custom_field_values: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub is_completed: bool,
    #[serde(default)]
    pub task_list_id: Option<String>,
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

