/// Maximum number of characters kept in a compact sanitized card description (find_cards, get_card).
const MAX_DESCRIPTION_CHARS: usize = 1500;

/// Minimum run length of base64-like chars before we treat it as a binary blob.
const MIN_BASE64_RUN: usize = 200;

/// Strip embedded images and base64 blobs from a card description, then truncate to 1500 chars.
///
/// Card descriptions on real Planka instances often contain inline `data:image/...;base64,...`
/// blobs (pasted screenshots) that blow up the agent's context window. We replace those with
/// short placeholders and cap the final length so reads stay cheap.
///
/// Use this for compact tools (find_cards, get_card). For full context use `sanitize_description_full`.
pub fn sanitize_description(raw: &str) -> String {
    sanitize_with_cap(raw, Some(MAX_DESCRIPTION_CHARS))
}

/// Strip embedded images and base64 blobs from a card description, but keep all text content.
///
/// Images pasted into Planka descriptions are stored as `data:image/...;base64,...` URIs which
/// are stripped and replaced with `[image omitted]`. This preserves all actual text content
/// (e.g. implementation plans) without an arbitrary character cap.
///
/// Use this for `get_card_context` where the full description is needed for context.
pub fn sanitize_description_full(raw: &str) -> String {
    sanitize_with_cap(raw, None)
}

fn sanitize_with_cap(raw: &str, max_chars: Option<usize>) -> String {
    let s = strip_data_uris(raw);
    let s = strip_long_base64_runs(&s);
    match max_chars {
        Some(max) => truncate_chars(&s, max),
        None => s,
    }
}

/// Replace `![alt](data:...)` markdown embeds and bare `data:image/...` URIs with `[image omitted]`.
fn strip_data_uris(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;

    loop {
        let next_md = rest.find("![");
        let next_data = rest.find("data:");

        let (pos, is_md) = match (next_md, next_data) {
            (None, None) => {
                out.push_str(rest);
                return out;
            }
            (Some(p), None) => (p, true),
            (None, Some(p)) => (p, false),
            (Some(a), Some(b)) => {
                if a <= b {
                    (a, true)
                } else {
                    (b, false)
                }
            }
        };

        out.push_str(&rest[..pos]);
        rest = &rest[pos..];

        if is_md {
            if let Some(bracket_paren) = rest.find("](") {
                let content_start = bracket_paren + 2;
                if rest[content_start..].starts_with("data:") {
                    if let Some(end_paren) = rest[content_start..].find(')') {
                        out.push_str("[image omitted]");
                        rest = &rest[content_start + end_paren + 1..];
                        continue;
                    }
                }
            }
            out.push_str("![");
            rest = &rest[2..];
        } else {
            let end = rest
                .find(|c: char| c.is_whitespace() || c == ')' || c == '"' || c == '\'')
                .unwrap_or(rest.len());
            out.push_str("[image omitted]");
            rest = &rest[end..];
        }
    }
}

/// Replace any remaining run of 200+ base64-like characters with `[binary omitted]`.
fn strip_long_base64_runs(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if is_base64_byte(bytes[i]) {
            let start = i;
            while i < bytes.len() && is_base64_byte(bytes[i]) {
                i += 1;
            }
            if i - start >= MIN_BASE64_RUN {
                out.push_str("[binary omitted]");
            } else {
                out.push_str(&input[start..i]);
            }
        } else {
            let ch_start = i;
            let ch = input[i..].chars().next().expect("char at byte boundary");
            i += ch.len_utf8();
            out.push_str(&input[ch_start..i]);
        }
    }

    out
}

fn is_base64_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'+' || b == b'/' || b == b'='
}

fn truncate_chars(input: &str, max_chars: usize) -> String {
    let total = input.chars().count();
    if total <= max_chars {
        return input.to_string();
    }
    let mut out: String = input.chars().take(max_chars).collect();
    out.push_str(&format!(
        "\n…[truncated, {} chars omitted]",
        total - max_chars
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_markdown_data_image_embed() {
        let input = "before ![pic](data:image/png;base64,AAAA) after";
        assert_eq!(sanitize_description(input), "before [image omitted] after");
    }

    #[test]
    fn strips_bare_data_uri() {
        let input = "see data:image/png;base64,AAAABBBB here";
        assert_eq!(sanitize_description(input), "see [image omitted] here");
    }

    #[test]
    fn keeps_normal_markdown_links_and_images() {
        let input = "![real](https://example.com/x.png) and [link](https://example.com)";
        assert_eq!(sanitize_description(input), input);
    }

    #[test]
    fn strips_long_base64_run() {
        let blob: String = "A".repeat(300);
        let input = format!("noise {blob} end");
        let out = sanitize_description(&input);
        assert!(out.contains("[binary omitted]"));
        assert!(!out.contains(&"A".repeat(200)));
    }

    #[test]
    fn keeps_short_alphanumeric_runs() {
        let input = "token=abc123 length=42";
        assert_eq!(sanitize_description(input), input);
    }

    #[test]
    fn truncates_when_too_long() {
        // Use realistic prose-shaped input (spaces break base64-run detection)
        let chunk = "lorem ipsum dolor sit amet ";
        let target_chars = MAX_DESCRIPTION_CHARS + 500;
        let repeats = target_chars / chunk.chars().count() + 1;
        let input: String = chunk.repeat(repeats);
        let out = sanitize_description(&input);
        assert!(out.contains("[truncated"), "expected truncation marker, got: {}", &out[out.len().saturating_sub(80)..]);
        assert!(out.chars().count() < input.chars().count());
    }

    #[test]
    fn handles_utf8_safely() {
        let input = "café — données data:image/png;base64,ZZZZ ✓";
        let out = sanitize_description(input);
        assert!(out.contains("café"));
        assert!(out.contains("[image omitted]"));
        assert!(out.contains("✓"));
    }
}
