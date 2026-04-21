use serde::Deserialize;
use serde_json::{json, Value};

use crate::mcp::types::{Tool, ToolAnnotations, ToolCallResult};
use crate::planka::PlankaClient;

/// Creates annotations enabling programmatic tool calling
fn programmatic_annotations() -> Option<ToolAnnotations> {
    Some(ToolAnnotations {
        allowed_callers: Some(vec!["code_execution_20250825".to_string()]),
    })
}

/// Returns the list of available tools
pub fn list_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "list_projects".to_string(),
            description: "Get all Planka projects with board counts. Use to discover available projects and their IDs.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of projects to return (default: 50)",
                        "default": 50
                    }
                },
                "required": []
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "list_board_summary".to_string(),
            description: "Get a board overview with lists and card counts. Use to understand board structure and find specific lists/cards.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {
                        "type": "string",
                        "description": "The board ID"
                    }
                },
                "required": ["board_id"]
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "find_cards".to_string(),
            description: "Search for cards on a board by name or list. Returns compact card summaries (id, name, list). Use to locate specific tasks.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {
                        "type": "string",
                        "description": "The board ID"
                    },
                    "query": {
                        "type": "string",
                        "description": "Optional search term to filter cards by name (case-insensitive)"
                    },
                    "list_id": {
                        "type": "string",
                        "description": "Optional list ID to filter cards by list"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of cards to return (default: 50)",
                        "default": 50
                    }
                },
                "required": ["board_id"]
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "get_card".to_string(),
            description: "Get full details of a specific card including complete description. Use after find_cards when you need the full content.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "card_id": {
                        "type": "string",
                        "description": "The card ID"
                    }
                },
                "required": ["card_id"]
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "create_card".to_string(),
            description: "Create a new task card in a list. Returns the created card ID.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "list_id": {
                        "type": "string",
                        "description": "The list ID to create the card in"
                    },
                    "name": {
                        "type": "string",
                        "description": "The card title (required)"
                    },
                    "description": {
                        "type": "string",
                        "description": "Optional card description"
                    },
                    "card_type": {
                        "type": "string",
                        "description": "Card type (e.g. 'task', 'project', 'story'). Defaults to PLANKA_DEFAULT_CARD_TYPE env var or 'task'."
                    }
                },
                "required": ["list_id", "name"]
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "update_card".to_string(),
            description: "Update a card's title or description.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "card_id": {
                        "type": "string",
                        "description": "The card ID to update"
                    },
                    "name": {
                        "type": "string",
                        "description": "New card title (optional)"
                    },
                    "description": {
                        "type": "string",
                        "description": "New card description (optional)"
                    }
                },
                "required": ["card_id"]
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "move_card".to_string(),
            description: "Move a card to a different list (e.g., from 'Todo' to 'In Progress').".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "card_id": {
                        "type": "string",
                        "description": "The card ID to move"
                    },
                    "list_id": {
                        "type": "string",
                        "description": "The target list ID"
                    }
                },
                "required": ["card_id", "list_id"]
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "add_comment".to_string(),
            description: "Post a comment on a card. Use to add summaries, status updates, or notes after completing work.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "card_id": {
                        "type": "string",
                        "description": "The card ID to comment on"
                    },
                    "text": {
                        "type": "string",
                        "description": "The comment text (supports Markdown)"
                    }
                },
                "required": ["card_id", "text"]
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "delete_card".to_string(),
            description: "Delete a card permanently (destructive operation - not recommended).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "card_id": {
                        "type": "string",
                        "description": "The card ID to delete"
                    }
                },
                "required": ["card_id"]
            }),
            annotations: None,
        },
    ]
}

/// Dispatch a tool call to the appropriate handler
pub async fn call_tool(client: &PlankaClient, name: &str, args: Option<Value>) -> ToolCallResult {
    match name {
        "list_projects" => list_projects(client, args).await,
        "list_board_summary" => list_board_summary(client, args).await,
        "find_cards" => find_cards(client, args).await,
        "create_card" => create_card(client, args).await,
        "update_card" => update_card(client, args).await,
        "move_card" => move_card(client, args).await,
        "get_card" => get_card(client, args).await,
        "add_comment" => add_comment(client, args).await,
        "delete_card" => delete_card(client, args).await,
        _ => ToolCallResult::error(format!("Unknown tool: {name}")),
    }
}

#[derive(Deserialize)]
struct ListProjectsArgs {
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    50
}

async fn list_projects(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: ListProjectsArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(_) => ListProjectsArgs { limit: 50 },
        },
        None => ListProjectsArgs { limit: 50 },
    };

    match client.list_projects().await {
        Ok(projects) => {
            let limited = projects.iter().take(args.limit).collect::<Vec<_>>();
            let compact: Vec<serde_json::Value> = limited
                .iter()
                .map(|p| {
                    json!({
                        "id": p.id,
                        "name": p.name
                    })
                })
                .collect();
            let json = serde_json::to_string_pretty(&compact).unwrap_or_default();
            ToolCallResult::text(json)
        }
        Err(e) => ToolCallResult::error(format!("Failed to list projects: {e}")),
    }
}

#[derive(Deserialize)]
struct BoardSummaryArgs {
    board_id: String,
}

async fn list_board_summary(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: BoardSummaryArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required argument: board_id"),
    };

    let lists = match client.list_lists(&args.board_id).await {
        Ok(l) => l,
        Err(e) => return ToolCallResult::error(format!("Failed to list lists: {e}")),
    };

    let cards = match client.list_cards(&args.board_id).await {
        Ok(c) => c,
        Err(e) => return ToolCallResult::error(format!("Failed to list cards: {e}")),
    };

    // Count cards per list
    use std::collections::HashMap;
    let mut card_counts: HashMap<String, usize> = HashMap::new();
    for card in &cards {
        *card_counts.entry(card.list_id.clone()).or_insert(0) += 1;
    }

    // Build compact response — skip archive/system lists (no name)
    let list_summaries: Vec<serde_json::Value> = lists
        .iter()
        .filter(|l| l.name.is_some())
        .map(|l| {
            json!({
                "id": l.id,
                "name": l.name,
                "card_count": card_counts.get(&l.id).copied().unwrap_or(0)
            })
        })
        .collect();

    let summary = json!({
        "lists": list_summaries,
        "total_cards": cards.len()
    });

    ToolCallResult::text(serde_json::to_string_pretty(&summary).unwrap_or_default())
}

#[derive(Deserialize)]
struct FindCardsArgs {
    board_id: String,
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    list_id: Option<String>,
    #[serde(default = "default_limit")]
    limit: usize,
}

async fn find_cards(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: FindCardsArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required argument: board_id"),
    };

    match client.list_cards(&args.board_id).await {
        Ok(cards) => {
            let mut filtered: Vec<_> = cards
                .iter()
                .filter(|c| {
                    // Filter by list_id if provided
                    if let Some(ref list_id) = args.list_id {
                        if c.list_id != *list_id {
                            return false;
                        }
                    }
                    // Filter by query (case-insensitive name search)
                    if let Some(ref query) = args.query {
                        if !c.name.to_lowercase().contains(&query.to_lowercase()) {
                            return false;
                        }
                    }
                    true
                })
                .collect();

            // Apply limit
            filtered.truncate(args.limit);

            // Return compact format
            let compact: Vec<serde_json::Value> = filtered
                .iter()
                .map(|c| {
                    json!({
                        "id": c.id,
                        "name": c.name,
                        "list_id": c.list_id,
                        "description": c.description.as_ref().map(|d| {
                            if d.len() > 200 {
                                format!("{}...", &d[..200])
                            } else {
                                d.clone()
                            }
                        })
                    })
                })
                .collect();

            let json = serde_json::to_string_pretty(&compact).unwrap_or_default();
            ToolCallResult::text(json)
        }
        Err(e) => ToolCallResult::error(format!("Failed to find cards: {e}")),
    }
}

#[derive(Deserialize)]
struct CreateCardArgs {
    list_id: String,
    name: String,
    description: Option<String>,
    card_type: Option<String>,
}

async fn create_card(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: CreateCardArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required arguments: list_id, name"),
    };

    let card_type = args.card_type
        .unwrap_or_else(|| std::env::var("PLANKA_DEFAULT_CARD_TYPE").unwrap_or_else(|_| "task".to_string()));

    match client
        .create_card(&args.list_id, &args.name, args.description.as_deref(), &card_type)
        .await
    {
        Ok(card) => {
            let result = json!({
                "id": card.id,
                "name": card.name,
                "list_id": card.list_id,
                "message": "Card created successfully"
            });
            ToolCallResult::text(serde_json::to_string_pretty(&result).unwrap_or_default())
        }
        Err(e) => ToolCallResult::error(format!("Failed to create card: {e}")),
    }
}

#[derive(Deserialize)]
struct UpdateCardArgs {
    card_id: String,
    name: Option<String>,
    description: Option<String>,
}

async fn update_card(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: UpdateCardArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required argument: card_id"),
    };

    match client
        .update_card(&args.card_id, args.name.as_deref(), args.description.as_deref())
        .await
    {
        Ok(card) => {
            let result = json!({
                "id": card.id,
                "name": card.name,
                "message": "Card updated successfully"
            });
            ToolCallResult::text(serde_json::to_string_pretty(&result).unwrap_or_default())
        }
        Err(e) => ToolCallResult::error(format!("Failed to update card: {e}")),
    }
}

#[derive(Deserialize)]
struct MoveCardArgs {
    card_id: String,
    list_id: String,
}

async fn move_card(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: MoveCardArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required arguments: card_id, list_id"),
    };

    match client.move_card(&args.card_id, &args.list_id, None).await {
        Ok(card) => {
            let result = json!({
                "id": card.id,
                "name": card.name,
                "list_id": card.list_id,
                "message": "Card moved successfully"
            });
            ToolCallResult::text(serde_json::to_string_pretty(&result).unwrap_or_default())
        }
        Err(e) => ToolCallResult::error(format!("Failed to move card: {e}")),
    }
}

#[derive(Deserialize)]
struct GetCardArgs {
    card_id: String,
}

async fn get_card(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: GetCardArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required argument: card_id"),
    };

    match client.get_card(&args.card_id).await {
        Ok(detail) => {
            let card = &detail.item;
            let tasks: Vec<serde_json::Value> = detail.included.tasks
                .iter()
                .map(|t| json!({
                    "name": t.name,
                    "done": t.is_completed
                }))
                .collect();

            let result = json!({
                "id": card.id,
                "name": card.name,
                "list_id": card.list_id,
                "description": card.description,
                "tasks": tasks
            });

            ToolCallResult::text(serde_json::to_string_pretty(&result).unwrap_or_default())
        }
        Err(e) => ToolCallResult::error(format!("Failed to get card: {e}")),
    }
}

#[derive(Deserialize)]
struct AddCommentArgs {
    card_id: String,
    text: String,
}

async fn add_comment(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: AddCommentArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required arguments: card_id, text"),
    };

    match client.add_comment(&args.card_id, &args.text).await {
        Ok(()) => ToolCallResult::text("Comment added successfully"),
        Err(e) => ToolCallResult::error(format!("Failed to add comment: {e}")),
    }
}

#[derive(Deserialize)]
struct DeleteCardArgs {
    card_id: String,
}

async fn delete_card(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: DeleteCardArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required argument: card_id"),
    };

    match client.delete_card(&args.card_id).await {
        Ok(()) => ToolCallResult::text("Card deleted successfully"),
        Err(e) => ToolCallResult::error(format!("Failed to delete card: {e}")),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tools_returns_all_tools() {
        let tools = list_tools();
        assert_eq!(tools.len(), 9, "Expected 9 tools");

        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"list_projects"));
        assert!(names.contains(&"list_board_summary"));
        assert!(names.contains(&"find_cards"));
        assert!(names.contains(&"get_card"));
        assert!(names.contains(&"create_card"));
        assert!(names.contains(&"update_card"));
        assert!(names.contains(&"move_card"));
        assert!(names.contains(&"add_comment"));
        assert!(names.contains(&"delete_card"));
    }

    #[test]
    fn test_programmatic_tools_have_allowed_callers() {
        let tools = list_tools();
        let programmatic_tools = [
            "list_projects",
            "list_board_summary",
            "find_cards",
            "get_card",
            "create_card",
            "update_card",
            "move_card",
            "add_comment",
        ];

        for tool_name in programmatic_tools {
            let tool = tools.iter().find(|t| t.name == tool_name).unwrap();
            let annotations = tool
                .annotations
                .as_ref()
                .unwrap_or_else(|| panic!("{tool_name} should have annotations"));
            let callers = annotations
                .allowed_callers
                .as_ref()
                .unwrap_or_else(|| panic!("{tool_name} should have allowed_callers"));
            assert!(
                callers.contains(&"code_execution_20250825".to_string()),
                "{tool_name} should allow code_execution_20250825"
            );
        }
    }

    #[test]
    fn test_delete_tool_excluded_from_programmatic_calling() {
        let tools = list_tools();
        let tool = tools.iter().find(|t| t.name == "delete_card").unwrap();
        assert!(
            tool.annotations.is_none(),
            "delete_card should NOT have annotations (destructive operation)"
        );
    }
}
