use base64::engine::general_purpose::{GeneralPurpose, STANDARD as BASE64};
use base64::engine::{DecodePaddingMode, GeneralPurposeConfig};
use base64::Engine as _;

/// Decoder for inline data URIs, which don't always carry canonical `=` padding.
const BASE64_FORGIVING: GeneralPurpose = GeneralPurpose::new(
    &base64::alphabet::STANDARD,
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
);
use serde::Deserialize;
use serde_json::{json, Value};

use crate::mcp::types::{Tool, ToolAnnotations, ToolCallResult, ToolContent};
use crate::planka::{
    extract_inline_images, sanitize_description, sanitize_description_full, PlankaClient,
};

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
            name: "get_card_context".to_string(),
            description: "FIRST CALL when the user gives you a Planka card URL. Extract the id from `/cards/{id}` and pass it as `card_id`. Returns the card plus its project, board, sibling lists (with names and ids), board labels, board members, plus the card's own labels/members/tasks/comments/attachments — enough to comment, move, update, or assign without any further discovery. Card description is fully returned with inline images stripped (data URIs replaced with [image omitted]) but NO character cap — full implementation plans and long descriptions are preserved.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "card_id": {
                        "type": "string",
                        "description": "The card ID (extract from a Planka URL like https://.../cards/{card_id})"
                    }
                },
                "required": ["card_id"]
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "get_card".to_string(),
            description: "Get a single card by id (name, sanitized description, list_id, tasks). Does NOT return sibling lists or board context — use `get_card_context` when you also need to know what columns exist on the board.".to_string(),
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
            name: "get_attachment".to_string(),
            description: "Fetch a card attachment's content. Images are returned inline (viewable), text files as text. Pass `card_id` plus `attachment_id` OR `attachment_name` (from the `attachments` array of `get_card`/`get_card_context`). Link attachments and unsupported binary types return their URL/metadata instead. Attachment download URLs require a logged-in browser session — this tool is the only way to read file contents.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "card_id": {
                        "type": "string",
                        "description": "The card ID the attachment belongs to"
                    },
                    "attachment_id": {
                        "type": "string",
                        "description": "The attachment ID. Provide this OR `attachment_name`, not both."
                    },
                    "attachment_name": {
                        "type": "string",
                        "description": "The attachment name (case-insensitive) on the card. Provide this OR `attachment_id`, not both."
                    }
                },
                "required": ["card_id"]
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "get_card_image".to_string(),
            description: "View an image pasted inline into a card's description. Sanitized descriptions replace embedded images with numbered placeholders like `[inline image #2: image/png, ~245 KB]` — pass that number as `index` to get the actual image back (viewable). For file attachments (the card's `attachments` array) use `get_attachment` instead.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "card_id": {
                        "type": "string",
                        "description": "The card ID"
                    },
                    "index": {
                        "type": "integer",
                        "description": "1-based inline image number, matching the `#N` in the description placeholder (default: 1)",
                        "default": 1
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
            description: "Update a card's title or description. Works with only `card_id` + the field(s) to change. Do NOT pre-fetch the card unless you need to read the existing description to merge edits.".to_string(),
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
            description: "Move a card to a different list. Pass `list_name` (e.g. \"Done\") when you only know the column by name — the server resolves it on the card's own board. Pass `list_id` if you already have it. Exactly one of the two is required. Do NOT pre-fetch the board just to translate a name.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "card_id": {
                        "type": "string",
                        "description": "The card ID to move"
                    },
                    "list_id": {
                        "type": "string",
                        "description": "The target list ID. Provide this OR `list_name`, not both."
                    },
                    "list_name": {
                        "type": "string",
                        "description": "Target list name (case-insensitive) on the card's own board. Provide this OR `list_id`, not both."
                    }
                },
                "required": ["card_id"]
            }),
            annotations: programmatic_annotations(),
        },
        Tool {
            name: "add_comment".to_string(),
            description: "Post a comment on a card. Works with only `card_id` + `text` — do NOT call `get_card` or `get_card_context` first unless you also need other context. Commenting does not require knowing the board.".to_string(),
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
        "get_card_context" => get_card_context(client, args).await,
        "get_attachment" => get_attachment(client, args).await,
        "get_card_image" => get_card_image(client, args).await,
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
                        "description": c.description.as_deref().map(|d| {
                            let cleaned = sanitize_description(d);
                            if cleaned.chars().count() > 200 {
                                let truncated: String = cleaned.chars().take(200).collect();
                                format!("{truncated}...")
                            } else {
                                cleaned
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
        .unwrap_or_else(|| std::env::var("PLANKA_DEFAULT_CARD_TYPE").unwrap_or_else(|_| "project".to_string()));

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
    #[serde(default)]
    list_id: Option<String>,
    #[serde(default)]
    list_name: Option<String>,
    #[serde(default)]
    position: Option<f64>,
}

async fn move_card(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: MoveCardArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required argument: card_id"),
    };

    let list_id = match (args.list_id.as_deref(), args.list_name.as_deref()) {
        (Some(_), Some(_)) => {
            return ToolCallResult::error(
                "Provide exactly one of list_id or list_name, not both",
            );
        }
        (None, None) => {
            return ToolCallResult::error("Provide list_id or list_name");
        }
        (Some(id), None) => id.to_string(),
        (None, Some(name)) => match resolve_list_id_by_name(client, &args.card_id, name).await {
            Ok(id) => id,
            Err(msg) => return ToolCallResult::error(msg),
        },
    };

    match client.move_card(&args.card_id, &list_id, args.position).await {
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

/// Look up a list id by name on the card's own board. Returns a JSON-RPC-friendly
/// error string that lists the available names if the lookup is unambiguous-or-missing.
async fn resolve_list_id_by_name(
    client: &PlankaClient,
    card_id: &str,
    list_name: &str,
) -> Result<String, String> {
    let target = list_name.trim().to_lowercase();
    if target.is_empty() {
        return Err("list_name is empty".to_string());
    }

    let detail = client
        .get_card(card_id)
        .await
        .map_err(|e| format!("Failed to look up card while resolving list_name: {e}"))?;
    let board_id = detail
        .item
        .board_id
        .as_ref()
        .ok_or_else(|| "Card has no board_id; cannot resolve list_name".to_string())?;

    let board = client
        .get_board(board_id)
        .await
        .map_err(|e| format!("Failed to fetch board while resolving list_name: {e}"))?;

    let named: Vec<&crate::planka::types::List> = board
        .included
        .lists
        .iter()
        .filter(|l| l.name.is_some())
        .collect();

    let matches: Vec<&&crate::planka::types::List> = named
        .iter()
        .filter(|l| {
            l.name
                .as_ref()
                .map(|n| n.trim().to_lowercase() == target)
                .unwrap_or(false)
        })
        .collect();

    match matches.len() {
        1 => Ok(matches[0].id.clone()),
        0 => {
            let available: Vec<&str> = named
                .iter()
                .filter_map(|l| l.name.as_deref())
                .collect();
            Err(format!(
                "No list named '{list_name}' on this board. Available lists: {}",
                available.join(", ")
            ))
        }
        _ => {
            let ids: Vec<&str> = matches.iter().map(|l| l.id.as_str()).collect();
            Err(format!(
                "Multiple lists named '{list_name}' on this board ({}). Pass list_id instead.",
                ids.join(", ")
            ))
        }
    }
}

/// Map a raw Planka attachment object to a compact JSON shape.
/// Planka v2 nests the download/link URL in `data.url` (with auth required for
/// file downloads); Planka v1 exposed it top-level, so fall back to that.
fn attachment_summary(a: &Value) -> Value {
    let data = a.get("data");
    let url = data
        .and_then(|d| d.get("url"))
        .or_else(|| a.get("url"))
        .cloned()
        .unwrap_or(Value::Null);
    json!({
        "id": a.get("id"),
        "name": a.get("name"),
        "type": a.get("type"),
        "url": url,
        "mime_type": data.and_then(|d| d.get("mimeType")),
        // Field name varies across Planka 2 releases: `sizeInBytes` vs `size`
        "size_in_bytes": data.and_then(|d| d.get("sizeInBytes").or_else(|| d.get("size"))),
        "created_at": a.get("createdAt")
    })
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

            let attachments: Vec<serde_json::Value> = detail.included.attachments
                .iter()
                .map(attachment_summary)
                .collect();

            let result = json!({
                "id": card.id,
                "name": card.name,
                "list_id": card.list_id,
                "description": card.description.as_deref().map(sanitize_description),
                "tasks": tasks,
                "attachments": attachments
            });

            ToolCallResult::text(serde_json::to_string_pretty(&result).unwrap_or_default())
        }
        Err(e) => ToolCallResult::error(format!("Failed to get card: {e}")),
    }
}

#[derive(Deserialize)]
struct GetCardContextArgs {
    card_id: String,
}

async fn get_card_context(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: GetCardContextArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required argument: card_id"),
    };

    let detail = match client.get_card(&args.card_id).await {
        Ok(d) => d,
        Err(e) => return ToolCallResult::error(format!("Failed to get card: {e}")),
    };
    let card = &detail.item;

    let board_id = match card.board_id.as_deref() {
        Some(id) => id,
        None => return ToolCallResult::error("Card has no board_id"),
    };

    // Fetch board and comments in parallel (comments failure is non-fatal)
    let (board_resp, comments_resp) = tokio::join!(
        client.get_board(board_id),
        client.get_comments(&args.card_id),
    );

    let board_resp = match board_resp {
        Ok(b) => b,
        Err(e) => return ToolCallResult::error(format!("Failed to get board: {e}")),
    };
    let comments_data = comments_resp.ok();
    let board = &board_resp.item;
    let included = &board_resp.included;

    let project_json = match board.project_id.as_deref() {
        Some(pid) => match client.get_project(pid).await {
            Ok(p) => json!({ "id": p.id, "name": p.name }),
            Err(_) => json!({ "id": pid }),
        },
        None => Value::Null,
    };

    let current_list = included
        .lists
        .iter()
        .find(|l| l.id == card.list_id)
        .map(|l| json!({ "id": l.id, "name": l.name }))
        .unwrap_or(Value::Null);

    let lists: Vec<Value> = included
        .lists
        .iter()
        .filter(|l| l.name.is_some())
        .map(|l| {
            json!({
                "id": l.id,
                "name": l.name,
                "position": l.position
            })
        })
        .collect();

    let label_by_id = |id: &str| -> Option<&Value> {
        included
            .labels
            .iter()
            .find(|l| l.get("id").and_then(|v| v.as_str()) == Some(id))
    };
    let user_by_id = |id: &str| -> Option<&Value> {
        included
            .users
            .iter()
            .chain(detail.included.users.iter())
            .find(|u| u.get("id").and_then(|v| v.as_str()) == Some(id))
    };

    let card_labels: Vec<Value> = detail
        .included
        .card_labels
        .iter()
        .filter_map(|cl| cl.get("labelId").and_then(|v| v.as_str()))
        .filter_map(label_by_id)
        .map(|l| {
            json!({
                "id": l.get("id"),
                "name": l.get("name"),
                "color": l.get("color")
            })
        })
        .collect();

    let card_members: Vec<Value> = detail
        .included
        .card_memberships
        .iter()
        .filter_map(|m| m.get("userId").and_then(|v| v.as_str()))
        .filter_map(user_by_id)
        .map(|u| {
            json!({
                "id": u.get("id"),
                "name": u.get("name"),
                "username": u.get("username")
            })
        })
        .collect();

    let tasks: Vec<Value> = detail
        .included
        .tasks
        .iter()
        .map(|t| {
            json!({
                "id": t.id,
                "name": t.name,
                "is_completed": t.is_completed
            })
        })
        .collect();

    // Resolve comments with author names
    let comments: Vec<Value> = comments_data
        .as_ref()
        .map(|cd| {
            cd.items.iter().map(|c| {
                let author_name = c.user_id.as_deref().and_then(|uid| {
                    cd.included.users.iter()
                        .chain(detail.included.users.iter())
                        .chain(included.users.iter())
                        .find(|u| u.get("id").and_then(|v| v.as_str()) == Some(uid))
                        .and_then(|u| u.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
                });
                json!({
                    "id": c.id,
                    "text": c.text,
                    "author": author_name,
                    "created_at": c.created_at
                })
            }).collect()
        })
        .unwrap_or_default();

    // Surface attachment metadata (name, type, download url; no binary content)
    let attachments: Vec<Value> = detail
        .included
        .attachments
        .iter()
        .map(attachment_summary)
        .collect();

    let board_labels: Vec<Value> = included
        .labels
        .iter()
        .map(|l| {
            json!({
                "id": l.get("id"),
                "name": l.get("name"),
                "color": l.get("color")
            })
        })
        .collect();

    let board_members: Vec<Value> = included
        .board_memberships
        .iter()
        .filter_map(|m| m.get("userId").and_then(|v| v.as_str()))
        .filter_map(user_by_id)
        .map(|u| {
            json!({
                "id": u.get("id"),
                "name": u.get("name"),
                "username": u.get("username")
            })
        })
        .collect();

    let response = json!({
        "card": {
            "id": card.id,
            "name": card.name,
            "description": card.description.as_deref().map(sanitize_description_full),
            "list_id": card.list_id,
            "position": card.position,
            "due_date": card.due_date,
            "labels": card_labels,
            "members": card_members,
            "tasks": tasks,
            "comments": comments,
            "attachments": attachments
        },
        "current_list": current_list,
        "board": {
            "id": board.id,
            "name": board.name,
            "project_id": board.project_id
        },
        "project": project_json,
        "lists": lists,
        "board_labels": board_labels,
        "board_members": board_members
    });

    ToolCallResult::text(serde_json::to_string_pretty(&response).unwrap_or_default())
}

#[derive(Deserialize)]
struct GetAttachmentArgs {
    card_id: String,
    #[serde(default)]
    attachment_id: Option<String>,
    #[serde(default)]
    attachment_name: Option<String>,
}

/// Raw image bytes above this are rejected — the Claude API caps images at 5 MB.
const MAX_IMAGE_BYTES: usize = 4 * 1024 * 1024;
/// Text attachments are truncated to this many characters.
const MAX_TEXT_CHARS: usize = 50_000;

fn is_text_mime(mime: &str) -> bool {
    mime.starts_with("text/")
        || mime.ends_with("+json")
        || mime.ends_with("+xml")
        || matches!(
            mime,
            "application/json"
                | "application/xml"
                | "application/yaml"
                | "application/x-yaml"
                | "application/javascript"
                | "application/csv"
                | "application/sql"
        )
}

async fn get_attachment(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: GetAttachmentArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required argument: card_id"),
    };

    let detail = match client.get_card(&args.card_id).await {
        Ok(d) => d,
        Err(e) => return ToolCallResult::error(format!("Failed to get card: {e}")),
    };
    let attachments = &detail.included.attachments;

    let matched: Vec<&Value> = match (args.attachment_id.as_deref(), args.attachment_name.as_deref()) {
        (Some(_), Some(_)) => {
            return ToolCallResult::error(
                "Provide exactly one of attachment_id or attachment_name, not both",
            );
        }
        (None, None) => {
            let available: Vec<String> = attachments
                .iter()
                .map(|a| {
                    format!(
                        "{} (id {})",
                        a.get("name").and_then(|v| v.as_str()).unwrap_or("?"),
                        a.get("id").and_then(|v| v.as_str()).unwrap_or("?")
                    )
                })
                .collect();
            return ToolCallResult::error(format!(
                "Provide attachment_id or attachment_name. Attachments on this card: {}",
                if available.is_empty() { "none".to_string() } else { available.join(", ") }
            ));
        }
        (Some(id), None) => attachments
            .iter()
            .filter(|a| a.get("id").and_then(|v| v.as_str()) == Some(id))
            .collect(),
        (None, Some(name)) => {
            let target = name.trim().to_lowercase();
            attachments
                .iter()
                .filter(|a| {
                    a.get("name")
                        .and_then(|v| v.as_str())
                        .map(|n| n.trim().to_lowercase() == target)
                        .unwrap_or(false)
                })
                .collect()
        }
    };

    let attachment = match matched.len() {
        1 => matched[0],
        0 => {
            let available: Vec<&str> = attachments
                .iter()
                .filter_map(|a| a.get("name").and_then(|v| v.as_str()))
                .collect();
            return ToolCallResult::error(format!(
                "No matching attachment on this card. Available attachments: {}",
                if available.is_empty() { "none".to_string() } else { available.join(", ") }
            ));
        }
        _ => {
            let ids: Vec<&str> = matched
                .iter()
                .filter_map(|a| a.get("id").and_then(|v| v.as_str()))
                .collect();
            return ToolCallResult::error(format!(
                "Multiple attachments match that name ({}). Pass attachment_id instead.",
                ids.join(", ")
            ));
        }
    };

    let summary = attachment_summary(attachment);
    let att_type = attachment
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("file");

    // Link attachments have nothing to download — return the URL itself.
    if att_type == "link" {
        return ToolCallResult::text(serde_json::to_string_pretty(&summary).unwrap_or_default());
    }

    let url = match summary.get("url").and_then(|v| v.as_str()) {
        Some(u) => u.to_string(),
        None => {
            return ToolCallResult::error(
                "Attachment has no download URL (unexpected Planka response shape)",
            );
        }
    };

    let (bytes, header_mime) = match client.download_attachment(&url).await {
        Ok(r) => r,
        Err(e) => return ToolCallResult::error(format!("Failed to download attachment: {e}")),
    };

    let mime = header_mime
        .filter(|m| !m.is_empty() && m != "application/octet-stream")
        .or_else(|| {
            attachment
                .get("data")
                .and_then(|d| d.get("mimeType"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "application/octet-stream".to_string());

    let meta = json!({
        "id": summary.get("id"),
        "name": summary.get("name"),
        "mime_type": mime,
        "size_in_bytes": bytes.len(),
        "url": url
    });
    let meta_text = serde_json::to_string_pretty(&meta).unwrap_or_default();

    if matches!(
        mime.as_str(),
        "image/png" | "image/jpeg" | "image/gif" | "image/webp"
    ) {
        if bytes.len() > MAX_IMAGE_BYTES {
            return ToolCallResult::error(format!(
                "Image is too large to return inline ({} bytes, limit {MAX_IMAGE_BYTES}). Open it in a browser instead: {url}",
                bytes.len()
            ));
        }
        return ToolCallResult {
            content: vec![
                ToolContent::Text { text: meta_text },
                ToolContent::Image {
                    data: BASE64.encode(&bytes),
                    mime_type: mime,
                },
            ],
            is_error: None,
        };
    }

    if is_text_mime(&mime) {
        let mut text = String::from_utf8_lossy(&bytes).into_owned();
        if text.chars().count() > MAX_TEXT_CHARS {
            text = text.chars().take(MAX_TEXT_CHARS).collect();
            text.push_str("\n... [truncated]");
        }
        return ToolCallResult {
            content: vec![
                ToolContent::Text { text: meta_text },
                ToolContent::Text { text },
            ],
            is_error: None,
        };
    }

    ToolCallResult::text(format!(
        "{meta_text}\n\nThis attachment type ({mime}) cannot be displayed inline. The user can open the URL above in a logged-in browser session."
    ))
}

#[derive(Deserialize)]
struct GetCardImageArgs {
    card_id: String,
    #[serde(default = "default_image_index")]
    index: usize,
}

fn default_image_index() -> usize {
    1
}

async fn get_card_image(client: &PlankaClient, args: Option<Value>) -> ToolCallResult {
    let args: GetCardImageArgs = match args {
        Some(v) => match serde_json::from_value(v) {
            Ok(a) => a,
            Err(e) => return ToolCallResult::error(format!("Invalid arguments: {e}")),
        },
        None => return ToolCallResult::error("Missing required argument: card_id"),
    };

    let detail = match client.get_card(&args.card_id).await {
        Ok(d) => d,
        Err(e) => return ToolCallResult::error(format!("Failed to get card: {e}")),
    };

    let raw_description = detail.item.description.as_deref().unwrap_or("");
    let images = extract_inline_images(raw_description);

    if images.is_empty() {
        return ToolCallResult::error(
            "This card's description contains no inline images. For file attachments use get_attachment.",
        );
    }
    if args.index == 0 || args.index > images.len() {
        return ToolCallResult::error(format!(
            "Inline image #{} does not exist — the description contains {} inline image(s). Pass an index between 1 and {}.",
            args.index,
            images.len(),
            images.len()
        ));
    }

    let image = &images[args.index - 1];

    let bytes = match BASE64_FORGIVING.decode(image.base64_data.as_bytes()) {
        Ok(b) => b,
        Err(e) => {
            return ToolCallResult::error(format!(
                "Inline image #{} has corrupt base64 data: {e}",
                args.index
            ));
        }
    };

    // Normalize the common non-standard alias
    let mime = if image.mime == "image/jpg" {
        "image/jpeg".to_string()
    } else {
        image.mime.clone()
    };

    let meta = json!({
        "card_id": detail.item.id,
        "index": args.index,
        "total_inline_images": images.len(),
        "mime_type": mime,
        "size_in_bytes": bytes.len()
    });
    let meta_text = serde_json::to_string_pretty(&meta).unwrap_or_default();

    if !matches!(
        mime.as_str(),
        "image/png" | "image/jpeg" | "image/gif" | "image/webp"
    ) {
        return ToolCallResult::text(format!(
            "{meta_text}\n\nInline data #{} is of type {mime}, which cannot be displayed as an image.",
            args.index
        ));
    }
    if bytes.len() > MAX_IMAGE_BYTES {
        return ToolCallResult::error(format!(
            "Inline image #{} is too large to return ({} bytes, limit {MAX_IMAGE_BYTES}).",
            args.index,
            bytes.len()
        ));
    }

    ToolCallResult {
        content: vec![
            ToolContent::Text { text: meta_text },
            ToolContent::Image {
                data: BASE64.encode(&bytes),
                mime_type: mime,
            },
        ],
        is_error: None,
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
        assert_eq!(tools.len(), 12, "Expected 12 tools");

        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"list_projects"));
        assert!(names.contains(&"list_board_summary"));
        assert!(names.contains(&"find_cards"));
        assert!(names.contains(&"get_card"));
        assert!(names.contains(&"get_card_context"));
        assert!(names.contains(&"get_attachment"));
        assert!(names.contains(&"get_card_image"));
        assert!(names.contains(&"create_card"));
        assert!(names.contains(&"update_card"));
        assert!(names.contains(&"move_card"));
        assert!(names.contains(&"add_comment"));
        assert!(names.contains(&"delete_card"));
    }

    #[test]
    fn test_attachment_summary_reads_planka_v2_nested_url() {
        let raw = json!({
            "id": "123",
            "name": "screenshot.png",
            "type": "file",
            "createdAt": "2026-07-07T10:00:00.000Z",
            "data": {
                "url": "https://kanban.local/attachments/123/download/screenshot.png",
                "mimeType": "image/png",
                "sizeInBytes": 4096,
                "thumbnailUrls": { "outside360": "https://kanban.local/..." }
            }
        });
        let summary = attachment_summary(&raw);
        assert_eq!(
            summary["url"],
            json!("https://kanban.local/attachments/123/download/screenshot.png")
        );
        assert_eq!(summary["type"], json!("file"));
        assert_eq!(summary["mime_type"], json!("image/png"));
        assert_eq!(summary["size_in_bytes"], json!(4096));
    }

    #[test]
    fn test_attachment_summary_falls_back_to_top_level_url() {
        let raw = json!({
            "id": "9",
            "name": "spec.pdf",
            "url": "https://kanban.local/attachments/9/download/spec.pdf"
        });
        let summary = attachment_summary(&raw);
        assert_eq!(
            summary["url"],
            json!("https://kanban.local/attachments/9/download/spec.pdf")
        );
    }

    #[test]
    fn test_attachment_summary_accepts_size_field_variant() {
        let raw = json!({
            "id": "5",
            "name": "report.xlsx",
            "type": "file",
            "data": { "size": 41582, "url": "https://kanban.local/attachments/5/download/report.xlsx" }
        });
        let summary = attachment_summary(&raw);
        assert_eq!(summary["size_in_bytes"], json!(41582));
    }

    #[test]
    fn test_is_text_mime() {
        assert!(is_text_mime("text/plain"));
        assert!(is_text_mime("text/csv"));
        assert!(is_text_mime("application/json"));
        assert!(is_text_mime("application/ld+json"));
        assert!(!is_text_mime("application/pdf"));
        assert!(!is_text_mime("image/png"));
    }

    #[test]
    fn test_programmatic_tools_have_allowed_callers() {
        let tools = list_tools();
        let programmatic_tools = [
            "list_projects",
            "list_board_summary",
            "find_cards",
            "get_card",
            "get_card_context",
            "get_attachment",
            "get_card_image",
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
