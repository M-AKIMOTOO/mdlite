const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const ITALIC: &str = "\x1b[3m";
const STRIKE: &str = "\x1b[9m";
const CYAN: &str = "\x1b[36m";
const BLUE: &str = "\x1b[94m";
const YELLOW: &str = "\x1b[93m";
const MAGENTA: &str = "\x1b[95m";

pub(crate) fn render_markdown(markdown: &str, color: bool) -> String {
    let normalized = markdown.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<&str> = normalized.lines().collect();
    let mut output = String::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_end();

        if trimmed.is_empty() {
            if !output.ends_with("\n\n") && !output.is_empty() {
                output.push('\n');
            }
            i += 1;
            continue;
        }

        if let Some((fence, info)) = parse_fence_start(trimmed) {
            i += 1;
            let mut code_lines = Vec::new();
            while i < lines.len() {
                let candidate = lines[i].trim_end();
                if is_fence_end(candidate, fence) {
                    i += 1;
                    break;
                }
                code_lines.push(lines[i]);
                i += 1;
            }
            render_code_block(&mut output, info, &code_lines, color);
            continue;
        }

        if let Some(level) = parse_atx_heading(trimmed) {
            render_heading(
                &mut output,
                level,
                parse_inline(
                    trimmed[level + 1..].trim().trim_end_matches('#').trim(),
                    color,
                ),
                color,
            );
            i += 1;
            continue;
        }

        if i + 1 < lines.len() {
            if let Some(level) = parse_setext_heading(lines[i + 1].trim()) {
                render_heading(&mut output, level, parse_inline(trimmed, color), color);
                i += 2;
                continue;
            }
        }

        if is_thematic_break(trimmed) {
            render_rule(&mut output, color);
            i += 1;
            continue;
        }

        if let Some((consumed, table)) = parse_table(&lines[i..], color) {
            output.push_str(&table);
            i += consumed;
            continue;
        }

        if let Some((consumed, blockquote)) = parse_blockquote(&lines[i..], color) {
            output.push_str(&blockquote);
            i += consumed;
            continue;
        }

        if let Some((consumed, list)) = parse_list(&lines[i..], color) {
            output.push_str(&list);
            i += consumed;
            continue;
        }

        let (consumed, paragraph) = parse_paragraph(&lines[i..], color);
        output.push_str(&paragraph);
        i += consumed;
    }

    if !output.ends_with('\n') {
        output.push('\n');
    }

    output
}

fn parse_fence_start(line: &str) -> Option<(char, &str)> {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix("```") {
        Some(('`', rest.trim()))
    } else if let Some(rest) = trimmed.strip_prefix("~~~") {
        Some(('~', rest.trim()))
    } else {
        None
    }
}

fn is_fence_end(line: &str, fence: char) -> bool {
    let trimmed = line.trim_start();
    match fence {
        '`' => trimmed.starts_with("```"),
        '~' => trimmed.starts_with("~~~"),
        _ => false,
    }
}

fn parse_atx_heading(line: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    let count = trimmed.chars().take_while(|&c| c == '#').count();
    if (1..=6).contains(&count) && trimmed.chars().nth(count) == Some(' ') {
        Some(count)
    } else {
        None
    }
}

fn parse_setext_heading(line: &str) -> Option<usize> {
    if !line.is_empty() && line.chars().all(|c| c == '=') {
        Some(1)
    } else if !line.is_empty() && line.chars().all(|c| c == '-') {
        Some(2)
    } else {
        None
    }
}

fn is_thematic_break(line: &str) -> bool {
    let stripped: String = line.chars().filter(|c| !c.is_whitespace()).collect();
    stripped.len() >= 3
        && (stripped.chars().all(|c| c == '-')
            || stripped.chars().all(|c| c == '*')
            || stripped.chars().all(|c| c == '_'))
}

fn parse_table(lines: &[&str], color: bool) -> Option<(usize, String)> {
    if lines.len() < 2 {
        return None;
    }

    let header = lines[0].trim();
    let divider = lines[1].trim();
    if !header.contains('|') || !is_table_divider(divider) {
        return None;
    }

    let mut rows = Vec::new();
    rows.push(split_table_row(header));

    let mut consumed = 2;
    while consumed < lines.len() {
        let line = lines[consumed].trim();
        if line.is_empty() || !line.contains('|') {
            break;
        }
        rows.push(split_table_row(line));
        consumed += 1;
    }

    Some((consumed, render_table(&rows, color)))
}

fn is_table_divider(line: &str) -> bool {
    if !line.contains('|') {
        return false;
    }
    split_table_row(line)
        .into_iter()
        .all(|cell| !cell.is_empty() && cell.chars().all(|c| c == ':' || c == '-'))
}

fn split_table_row(line: &str) -> Vec<String> {
    let trimmed = line.trim().trim_matches('|');
    trimmed
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

fn render_table(rows: &[Vec<String>], color: bool) -> String {
    let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
    let mut widths = vec![0; columns];

    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            widths[index] = widths[index].max(strip_markers(cell).chars().count());
        }
    }

    let mut out = String::new();
    for (row_index, row) in rows.iter().enumerate() {
        out.push('|');
        for (index, width) in widths.iter().enumerate() {
            let cell = row.get(index).map(String::as_str).unwrap_or("");
            let rendered = parse_inline(cell, color);
            out.push(' ');
            if row_index == 0 && color {
                out.push_str(BOLD);
                out.push_str(&rendered);
                out.push_str(RESET);
            } else {
                out.push_str(&rendered);
            }
            let padding = width.saturating_sub(strip_markers(cell).chars().count());
            for _ in 0..padding {
                out.push(' ');
            }
            out.push(' ');
            out.push('|');
        }
        out.push('\n');
        if row_index == 0 {
            out.push('|');
            for width in &widths {
                out.push_str(&"-".repeat(*width + 2));
                out.push('|');
            }
            out.push('\n');
        }
    }
    out.push('\n');
    out
}

fn parse_blockquote(lines: &[&str], color: bool) -> Option<(usize, String)> {
    if !lines[0].trim_start().starts_with('>') {
        return None;
    }

    let mut consumed = 0;
    let mut parts = Vec::new();
    while consumed < lines.len() {
        let line = lines[consumed];
        if let Some(rest) = line.trim_start().strip_prefix('>') {
            parts.push(rest.trim_start());
            consumed += 1;
        } else if line.trim().is_empty() {
            parts.push("");
            consumed += 1;
        } else {
            break;
        }
    }

    let nested = render_markdown(&parts.join("\n"), false);
    let mut out = String::new();
    for line in nested.lines() {
        if color {
            out.push_str(DIM);
            out.push_str("| ");
            out.push_str(RESET);
        } else {
            out.push_str("| ");
        }
        out.push_str(line);
        out.push('\n');
    }
    out.push('\n');
    Some((consumed, out))
}

fn parse_list(lines: &[&str], color: bool) -> Option<(usize, String)> {
    let mut consumed = 0;
    let mut out = String::new();

    while consumed < lines.len() {
        let line = lines[consumed];
        if line.trim().is_empty() {
            break;
        }

        let Some(item) = parse_list_item(line, color) else {
            break;
        };

        out.push_str(&item);
        out.push('\n');
        consumed += 1;
    }

    if consumed == 0 {
        None
    } else {
        out.push('\n');
        Some((consumed, out))
    }
}

fn parse_list_item(line: &str, color: bool) -> Option<String> {
    let trimmed = line.trim_start();
    let content = if let Some(rest) = trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))
        .or_else(|| trimmed.strip_prefix("+ "))
    {
        rest
    } else {
        let digits = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
        if digits > 0 && trimmed[digits..].starts_with(". ") {
            &trimmed[digits + 2..]
        } else {
            return None;
        }
    };

    let (marker, rest) = if let Some(rest) = content.strip_prefix("[ ] ") {
        ("[ ]", rest)
    } else if let Some(rest) = content
        .strip_prefix("[x] ")
        .or_else(|| content.strip_prefix("[X] "))
    {
        ("[x]", rest)
    } else {
        ("-", content)
    };

    let mut item = String::new();
    if color {
        item.push_str(CYAN);
        item.push_str(marker);
        item.push_str(RESET);
        item.push(' ');
    } else {
        item.push_str(marker);
        item.push(' ');
    }
    item.push_str(&parse_inline(rest, color));
    Some(item)
}

fn parse_paragraph(lines: &[&str], color: bool) -> (usize, String) {
    let mut consumed = 0;
    let mut parts = Vec::new();

    while consumed < lines.len() {
        let line = lines[consumed];
        let trimmed = line.trim_end();
        if trimmed.is_empty()
            || parse_fence_start(trimmed).is_some()
            || parse_atx_heading(trimmed).is_some()
            || is_thematic_break(trimmed)
            || parse_list_item(trimmed, false).is_some()
            || trimmed.trim_start().starts_with('>')
            || (consumed + 1 < lines.len()
                && parse_setext_heading(lines[consumed + 1].trim()).is_some())
        {
            break;
        }
        parts.push(trimmed.trim());
        consumed += 1;
    }

    let mut out = parse_inline(&parts.join(" "), color);
    out.push_str("\n\n");
    (consumed.max(1), out)
}

fn render_heading(out: &mut String, level: usize, content: String, color: bool) {
    if color {
        out.push_str(BOLD);
        out.push_str(match level {
            1 => YELLOW,
            2 => BLUE,
            _ => MAGENTA,
        });
        out.push_str(&content);
        out.push_str(RESET);
    } else {
        out.push_str(&content);
    }
    out.push('\n');
    out.push('\n');
}

fn render_code_block(out: &mut String, _info: &str, code_lines: &[&str], color: bool) {
    for line in code_lines {
        if color {
            out.push_str(DIM);
            out.push_str("  ");
            out.push_str(line);
            out.push_str(RESET);
        } else {
            out.push_str("  ");
            out.push_str(line);
        }
        out.push('\n');
    }
    out.push('\n');
}

fn render_rule(out: &mut String, color: bool) {
    if color {
        out.push_str(DIM);
        out.push_str("----------------------------------------");
        out.push_str(RESET);
    } else {
        out.push_str("----------------------------------------");
    }
    out.push_str("\n\n");
}

fn parse_inline(text: &str, color: bool) -> String {
    let mut out = String::new();
    let bytes = text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'`' {
            if let Some(end) = text[i + 1..].find('`') {
                let code = &text[i + 1..i + 1 + end];
                if color {
                    out.push_str(BOLD);
                    out.push_str(CYAN);
                    out.push_str(code);
                    out.push_str(RESET);
                } else {
                    out.push_str(code);
                }
                i += end + 2;
                continue;
            }
        }

        if text[i..].starts_with("~~") {
            if let Some(end) = text[i + 2..].find("~~") {
                let inner = parse_inline(&text[i + 2..i + 2 + end], color);
                if color {
                    out.push_str(STRIKE);
                    out.push_str(&inner);
                    out.push_str(RESET);
                } else {
                    out.push_str(&inner);
                }
                i += end + 4;
                continue;
            }
        }

        if text[i..].starts_with("**") {
            if let Some(end) = text[i + 2..].find("**") {
                let inner = parse_inline(&text[i + 2..i + 2 + end], color);
                if color {
                    out.push_str(BOLD);
                    out.push_str(&inner);
                    out.push_str(RESET);
                } else {
                    out.push_str(&inner);
                }
                i += end + 4;
                continue;
            }
        }

        if text[i..].starts_with('*') {
            if let Some(end) = text[i + 1..].find('*') {
                let inner = parse_inline(&text[i + 1..i + 1 + end], color);
                if color {
                    out.push_str(ITALIC);
                    out.push_str(&inner);
                    out.push_str(RESET);
                } else {
                    out.push_str(&inner);
                }
                i += end + 2;
                continue;
            }
        }

        if bytes[i] == b'[' {
            if let Some(mid) = text[i + 1..].find("](") {
                let text_end = i + 1 + mid;
                if let Some(url_end) = text[text_end + 2..].find(')') {
                    let label = parse_inline(&text[i + 1..text_end], color);
                    let url = &text[text_end + 2..text_end + 2 + url_end];
                    out.push_str(&label);
                    if color {
                        out.push_str(DIM);
                        out.push_str(" <");
                        out.push_str(url);
                        out.push('>');
                        out.push_str(RESET);
                    } else {
                        out.push_str(" <");
                        out.push_str(url);
                        out.push('>');
                    }
                    i = text_end + 3 + url_end;
                    continue;
                }
            }
        }

        let ch = text[i..].chars().next().expect("valid char boundary");
        out.push(ch);
        i += ch.len_utf8();
    }

    out
}

fn strip_markers(text: &str) -> String {
    text.replace("**", "")
        .replace('*', "")
        .replace("~~", "")
        .replace('`', "")
}

#[cfg(test)]
mod tests {
    use super::render_markdown;

    #[test]
    fn renders_headings_and_paragraphs() {
        let output = render_markdown("# Title\n\nhello world\n", false);
        assert!(output.contains("Title"));
        assert!(output.contains("hello world"));
    }

    #[test]
    fn renders_fenced_code_blocks() {
        let output = render_markdown("```rust\nfn main() {}\n```\n", false);
        assert!(output.contains("fn main() {}"));
        assert!(!output.contains("rust\n"));
    }

    #[test]
    fn renders_lists_and_tasks() {
        let output = render_markdown("- item\n- [x] done\n", false);
        assert!(output.contains("- item"));
        assert!(output.contains("[x] done"));
    }

    #[test]
    fn renders_blockquotes() {
        let output = render_markdown("> quoted\n> line\n", false);
        assert!(output.contains("| quoted line"));
    }

    #[test]
    fn renders_tables() {
        let output = render_markdown("| a | b |\n|---|---|\n| 1 | 2 |\n", false);
        assert!(output.contains("| a "));
        assert!(output.contains("| 1 "));
    }

    #[test]
    fn renders_inline_markup() {
        let output = render_markdown("**bold** *it* ~~gone~~ `code` [x](https://x)\n", false);
        assert!(output.contains("bold"));
        assert!(output.contains("it"));
        assert!(output.contains("gone"));
        assert!(output.contains("code"));
        assert!(output.contains("x <https://x>"));
    }
}
