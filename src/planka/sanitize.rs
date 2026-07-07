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

/// An inline `data:*;base64,*` payload extracted from a card description.
pub struct InlineImage {
    pub mime: String,
    pub base64_data: String,
}

/// Extract all inline data URIs from a raw description, in document order.
///
/// The 1-based position in this Vec matches the `#N` in the placeholders that
/// `sanitize_description`/`sanitize_description_full` emit — both use the same
/// parser, so `get_card_image(index: N)` always resolves to placeholder `#N`.
pub fn extract_inline_images(raw: &str) -> Vec<InlineImage> {
    let mut images = Vec::new();
    let mut rest = raw;
    while let Some(pos) = rest.find("data:") {
        rest = &rest[pos..];
        match parse_data_uri(rest) {
            Some((consumed, mime, payload)) => {
                images.push(InlineImage {
                    mime: mime.to_string(),
                    base64_data: payload.to_string(),
                });
                rest = &rest[consumed..];
            }
            None => rest = &rest[5..], // skip the "data:" literal, keep scanning
        }
    }
    images
}

/// Parse a `data:<mime>;base64,<payload>` URI at the start of `s`.
/// Returns (bytes consumed, mime, payload) or None if it isn't a valid data URI.
fn parse_data_uri(s: &str) -> Option<(usize, &str, &str)> {
    let rest = s.strip_prefix("data:")?;
    let semi = rest.find(";base64,")?;
    let mime = &rest[..semi];
    if mime.is_empty()
        || mime.len() > 100
        || !mime.contains('/')
        || !mime
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'+' | b'.'))
    {
        return None;
    }
    let payload_start = semi + ";base64,".len();
    let payload_rest = &rest[payload_start..];
    let end = payload_rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='))
        .unwrap_or(payload_rest.len());
    if end == 0 {
        return None;
    }
    Some((5 + payload_start + end, mime, &payload_rest[..end]))
}

fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{} KB", bytes / 1024)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Replace inline data URIs (markdown-wrapped or bare) with numbered placeholders
/// like `[inline image #1: image/png, ~245 KB]` so the agent can fetch them
/// on demand via `get_card_image`.
fn strip_data_uris(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    let mut index = 0usize;

    while let Some(pos) = rest.find("data:") {
        let mut text_end = pos;
        match parse_data_uri(&rest[pos..]) {
            Some((consumed, mime, payload)) => {
                let mut uri_end = pos + consumed;

                // If wrapped in a markdown image `![alt](data:...)`, swallow the
                // whole embed, not just the URI.
                if rest[..pos].ends_with("](") {
                    if let Some(bang) = rest[..pos - 2].rfind("![") {
                        let alt = &rest[bang + 2..pos - 2];
                        if !alt.contains(']') && !alt.contains('\n') {
                            text_end = bang;
                            if rest[uri_end..].starts_with(')') {
                                uri_end += 1;
                            }
                        }
                    }
                }

                index += 1;
                let kind = if mime.starts_with("image/") { "image" } else { "file" };
                let size = format_size(payload.len() / 4 * 3);
                out.push_str(&rest[..text_end]);
                out.push_str(&format!("[inline {kind} #{index}: {mime}, ~{size}]"));
                rest = &rest[uri_end..];
            }
            None => {
                // Not a real data URI — keep the text and continue after "data:".
                out.push_str(&rest[..pos + 5]);
                rest = &rest[pos + 5..];
            }
        }
    }

    out.push_str(rest);
    out
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
        assert_eq!(
            sanitize_description(input),
            "before [inline image #1: image/png, ~3 B] after"
        );
    }

    #[test]
    fn strips_bare_data_uri() {
        let input = "see data:image/png;base64,AAAABBBB here";
        assert_eq!(
            sanitize_description(input),
            "see [inline image #1: image/png, ~6 B] here"
        );
    }

    #[test]
    fn numbers_multiple_inline_images_in_order() {
        let input = "a ![x](data:image/png;base64,AAAA) b data:image/jpeg;base64,BBBBCCCC c";
        let out = sanitize_description(input);
        assert!(out.contains("[inline image #1: image/png"), "got: {out}");
        assert!(out.contains("[inline image #2: image/jpeg"), "got: {out}");
    }

    #[test]
    fn labels_non_image_data_uris_as_files() {
        let input = "doc data:application/pdf;base64,AAAA end";
        let out = sanitize_description(input);
        assert!(out.contains("[inline file #1: application/pdf"), "got: {out}");
    }

    #[test]
    fn keeps_plain_data_colon_text() {
        let input = "Update data: none yet";
        assert_eq!(sanitize_description(input), input);
    }

    #[test]
    fn extraction_order_matches_placeholder_numbers() {
        let input = "![a](data:image/png;base64,AAAA) mid data:image/webp;base64,BBBB end";
        let images = extract_inline_images(input);
        assert_eq!(images.len(), 2);
        assert_eq!(images[0].mime, "image/png");
        assert_eq!(images[0].base64_data, "AAAA");
        assert_eq!(images[1].mime, "image/webp");
        assert_eq!(images[1].base64_data, "BBBB");

        let out = sanitize_description(input);
        assert!(out.contains("#1: image/png"), "got: {out}");
        assert!(out.contains("#2: image/webp"), "got: {out}");
    }

    #[test]
    fn extraction_ignores_invalid_data_uris() {
        let images = extract_inline_images("data: nope, data:image/png;base64,QQQQ yes");
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].base64_data, "QQQQ");
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
        assert!(out.contains("[inline image #1: image/png"));
        assert!(out.contains("✓"));
    }
}
