//! Markdown parsing utilities for extracting metadata from markdown files.
//!
//! Handles:
//! - YAML frontmatter (Obsidian, Hugo, Jekyll style)
//! - Heading extraction
//! - Wiki-style links `[[link]]`
//! - Code block extraction with language tags
//! - Markdown syntax stripping

use std::collections::HashSet;

/// Metadata extracted from a markdown file
#[derive(Debug, Default, Clone)]
pub struct MarkdownMeta {
    /// Title from frontmatter or first H1
    pub title: Option<String>,
    /// Tags from frontmatter
    pub tags: Vec<String>,
    /// Wiki-style links found in the document
    pub links: Vec<String>,
    /// Headings with their levels (1-6)
    pub headings: Vec<Heading>,
    /// Code blocks with their language tags
    pub code_blocks: Vec<CodeBlock>,
}

/// A heading extracted from markdown
#[derive(Debug, Clone)]
pub struct Heading {
    pub level: u8,
    pub text: String,
}

/// A fenced code block extracted from markdown
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CodeBlock {
    /// Language tag (e.g., "rust", "python", "javascript")
    pub language: Option<String>,
    /// The code content
    pub content: String,
}

impl MarkdownMeta {
    /// Convert tags to JSON string for storage
    #[must_use]
    pub fn tags_json(&self) -> String {
        serde_json::to_string(&self.tags).unwrap_or_else(|_| "[]".to_string())
    }

    /// Convert links to JSON string for storage
    #[must_use]
    pub fn links_json(&self) -> String {
        serde_json::to_string(&self.links).unwrap_or_else(|_| "[]".to_string())
    }

    /// Convert headings to JSON string for storage
    #[must_use]
    pub fn headings_json(&self) -> String {
        let heading_strs: Vec<String> = self
            .headings
            .iter()
            .map(|h| format!("h{}:{}", h.level, h.text))
            .collect();
        serde_json::to_string(&heading_strs).unwrap_or_else(|_| "[]".to_string())
    }

    /// Convert code blocks to JSON string for storage
    #[must_use]
    #[allow(dead_code)]
    pub fn code_blocks_json(&self) -> String {
        let block_strs: Vec<String> = self
            .code_blocks
            .iter()
            .map(|b| {
                let lang = b.language.as_deref().unwrap_or("text");
                format!("{}:{}", lang, b.content.lines().count())
            })
            .collect();
        serde_json::to_string(&block_strs).unwrap_or_else(|_| "[]".to_string())
    }
}

/// Parse markdown content and extract metadata
#[must_use]
pub fn parse_markdown(content: &str) -> MarkdownMeta {
    parse_markdown_with_options(content, true)
}

/// Parse markdown content with options for code block extraction
#[must_use]
pub fn parse_markdown_with_options(content: &str, extract_code: bool) -> MarkdownMeta {
    let mut meta = MarkdownMeta::default();

    // Parse frontmatter if present
    if let Some(frontmatter) = extract_frontmatter(content) {
        parse_frontmatter(&frontmatter, &mut meta);
    }

    // Extract headings
    meta.headings = extract_headings(content);

    // If no title from frontmatter, use first H1
    if meta.title.is_none() {
        if let Some(h1) = meta.headings.iter().find(|h| h.level == 1) {
            meta.title = Some(h1.text.clone());
        }
    }

    // Extract wiki-style links
    meta.links = extract_wiki_links(content);

    // Extract code blocks if requested
    if extract_code {
        meta.code_blocks = extract_code_blocks(content);
    }

    meta
}

/// Extract YAML frontmatter from markdown content
fn extract_frontmatter(content: &str) -> Option<String> {
    let content = content.trim_start();

    // Must start with ---
    if !content.starts_with("---") {
        return None;
    }

    // Find the closing ---
    let after_opening = &content[3..];
    let closing_pos = after_opening.find("\n---")?;

    Some(after_opening[..closing_pos].trim().to_string())
}

/// Parse YAML frontmatter and populate metadata
fn parse_frontmatter(frontmatter: &str, meta: &mut MarkdownMeta) {
    for line in frontmatter.lines() {
        let line = line.trim();

        // Parse title: value
        if let Some(value) = line.strip_prefix("title:") {
            let value = value.trim().trim_matches('"').trim_matches('\'');
            if !value.is_empty() {
                meta.title = Some(value.to_string());
            }
        }

        // Parse tags: [tag1, tag2] or tags:\n  - tag1
        if let Some(value) = line.strip_prefix("tags:") {
            let value = value.trim();
            if value.starts_with('[') && value.ends_with(']') {
                // Inline array format: [tag1, tag2]
                let inner = &value[1..value.len() - 1];
                for tag in inner.split(',') {
                    let tag = tag.trim().trim_matches('"').trim_matches('\'');
                    if !tag.is_empty() {
                        meta.tags.push(tag.to_string());
                    }
                }
            }
        }

        // Parse YAML list item for tags
        if line.starts_with("- ") && meta.tags.is_empty() {
            // This might be a tag in list format, but we need context
            // For simplicity, we'll handle inline format primarily
        }
    }

    // Also try parsing as YAML for more complex cases
    if meta.tags.is_empty() {
        if let Some(tags) = parse_yaml_tags(frontmatter) {
            meta.tags = tags;
        }
    }
}

/// Try to parse tags from YAML frontmatter using simple pattern matching
fn parse_yaml_tags(frontmatter: &str) -> Option<Vec<String>> {
    let mut tags = Vec::new();
    let mut in_tags_section = false;

    for line in frontmatter.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("tags:") {
            in_tags_section = true;
            // Check for inline value
            let value = trimmed.strip_prefix("tags:")?.trim();
            if value.starts_with('[') && value.ends_with(']') {
                let inner = &value[1..value.len() - 1];
                for tag in inner.split(',') {
                    let tag = tag.trim().trim_matches('"').trim_matches('\'');
                    if !tag.is_empty() {
                        tags.push(tag.to_string());
                    }
                }
                return Some(tags);
            }
            continue;
        }

        if in_tags_section {
            // Check if we're still in the tags list
            if trimmed.starts_with("- ") {
                let tag = trimmed
                    .strip_prefix("- ")?
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                if !tag.is_empty() {
                    tags.push(tag.to_string());
                }
            } else if !trimmed.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                // New top-level key, exit tags section
                break;
            }
        }
    }

    if tags.is_empty() {
        None
    } else {
        Some(tags)
    }
}

/// Extract headings from markdown content
fn extract_headings(content: &str) -> Vec<Heading> {
    let mut headings = Vec::new();

    // Skip frontmatter if present
    let content = skip_frontmatter(content);

    for line in content.lines() {
        let trimmed = line.trim_start();

        // ATX-style headings: # Heading
        if trimmed.starts_with('#') {
            let mut level = 0u8;
            for ch in trimmed.chars() {
                if ch == '#' {
                    level += 1;
                } else {
                    break;
                }
            }

            if (1..=6).contains(&level) {
                let text = trimmed[level as usize..]
                    .trim_start()
                    .trim_end_matches('#')
                    .trim();
                if !text.is_empty() {
                    headings.push(Heading {
                        level,
                        text: text.to_string(),
                    });
                }
            }
        }
    }

    headings
}

/// Extract wiki-style links from markdown content
fn extract_wiki_links(content: &str) -> Vec<String> {
    let mut links = HashSet::new();
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Look for [[
        if chars[i] == '[' && i + 1 < chars.len() && chars[i + 1] == '[' {
            i += 2; // skip [[

            let mut link = String::new();
            let mut found_closing = false;

            while i < chars.len() {
                let ch = chars[i];

                // Check for closing ]]
                if ch == ']' && i + 1 < chars.len() && chars[i + 1] == ']' {
                    i += 2; // skip ]]
                    found_closing = true;
                    break;
                }

                // Stop at pipe (for [[target|display]] format)
                if ch == '|' {
                    // Skip until ]]
                    while i < chars.len() {
                        if chars[i] == ']' && i + 1 < chars.len() && chars[i + 1] == ']' {
                            i += 2;
                            found_closing = true;
                            break;
                        }
                        i += 1;
                    }
                    break;
                }

                // Links don't span lines
                if ch == '\n' {
                    break;
                }

                link.push(ch);
                i += 1;
            }

            if found_closing {
                let link = link.trim();
                if !link.is_empty() {
                    links.insert(link.to_string());
                }
            }
        } else {
            i += 1;
        }
    }

    let mut result: Vec<_> = links.into_iter().collect();
    result.sort();
    result
}

/// Skip frontmatter and return content after it
fn skip_frontmatter(content: &str) -> &str {
    let content = content.trim_start();

    if !content.starts_with("---") {
        return content;
    }

    let after_opening = &content[3..];
    if let Some(closing_pos) = after_opening.find("\n---") {
        // Return content after the closing ---
        let after_closing = &after_opening[closing_pos + 4..];
        after_closing.trim_start_matches('\n')
    } else {
        content
    }
}

/// Extract fenced code blocks from markdown content
fn extract_code_blocks(content: &str) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let content = skip_frontmatter(content);
    let lines: Vec<&str> = content.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        // Check for fenced code block start (``` or ~~~)
        let fence = if line.starts_with("```") {
            Some("```")
        } else if line.starts_with("~~~") {
            Some("~~~")
        } else {
            None
        };

        if let Some(fence_char) = fence {
            let language = line.strip_prefix(fence_char)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.split_whitespace().next().unwrap_or(s).to_string());

            i += 1;
            let mut code_content = String::new();

            // Collect lines until closing fence
            while i < lines.len() {
                let code_line = lines[i];
                if code_line.trim().starts_with(fence_char) {
                    break;
                }
                if !code_content.is_empty() {
                    code_content.push('\n');
                }
                code_content.push_str(code_line);
                i += 1;
            }

            if !code_content.is_empty() {
                blocks.push(CodeBlock {
                    language,
                    content: code_content,
                });
            }
        }
        i += 1;
    }

    blocks
}

/// Strip markdown syntax from content for cleaner full-text search.
/// Removes:
/// - Frontmatter
/// - Headers markers (#)
/// - Bold/italic markers (** * __)
/// - Links (keeps text)
/// - Code block fences (keeps code)
/// - HTML tags
/// - Blockquote markers (>)
#[must_use]
#[allow(dead_code)]
pub fn strip_markdown_syntax(content: &str) -> String {
    let content = skip_frontmatter(content);
    let mut result = String::with_capacity(content.len());
    let lines: Vec<&str> = content.lines().collect();

    let mut in_code_block = false;

    for line in lines {
        // Handle code block boundaries
        if line.trim().starts_with("```") || line.trim().starts_with("~~~") {
            in_code_block = !in_code_block;
            result.push('\n');
            continue;
        }

        // Keep code block content as-is
        if in_code_block {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        let processed = strip_line(line);
        result.push_str(&processed);
        result.push('\n');
    }

    result
}

/// Strip markdown syntax from a single line
#[allow(dead_code)]
fn strip_line(line: &str) -> String {
    let mut result = line.to_string();

    // Remove heading markers
    if result.trim_start().starts_with('#') {
        let trimmed = result.trim_start();
        let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
        if hash_count <= 6 {
            result = trimmed[hash_count..].trim_start().to_string();
        }
    }

    // Remove blockquote markers
    while result.trim_start().starts_with('>') {
        let trimmed = result.trim_start();
        result = trimmed[1..].trim_start().to_string();
    }

    // Remove bold/italic markers
    result = result.replace("**", "");
    result = result.replace("__", "");
    // Remove single asterisks/underscores that aren't part of words (simplified)

    // Remove inline code backticks
    result = result.replace('`', "");

    // Remove strikethrough
    result = result.replace("~~", "");

    // Convert markdown links [text](url) to just text
    result = strip_markdown_links(&result);

    // Convert wiki links [[link|display]] to display, [[link]] to link
    result = strip_wiki_links(&result);

    // Remove image syntax ![alt](url)
    result = strip_images(&result);

    // Remove horizontal rules
    let trimmed = result.trim();
    if trimmed == "---" || trimmed == "***" || trimmed == "___" {
        result = String::new();
    }

    result
}

/// Strip markdown links \[text\](url) -> text
#[allow(dead_code)]
fn strip_markdown_links(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '[' {
            // Look for ]( pattern
            let mut j = i + 1;
            let mut link_text = String::new();

            while j < chars.len() && chars[j] != ']' && chars[j] != '\n' {
                link_text.push(chars[j]);
                j += 1;
            }

            if j < chars.len() && chars[j] == ']' && j + 1 < chars.len() && chars[j + 1] == '(' {
                // Found [text](, now skip until )
                let mut k = j + 2;
                while k < chars.len() && chars[k] != ')' && chars[k] != '\n' {
                    k += 1;
                }
                if k < chars.len() && chars[k] == ')' {
                    result.push_str(&link_text);
                    i = k + 1;
                    continue;
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Strip wiki links \[\[link|display\]\] -> display, \[\[link\]\] -> link
#[allow(dead_code)]
fn strip_wiki_links(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '[' && chars[i + 1] == '[' {
            i += 2;
            let mut link = String::new();
            let mut display = String::new();
            let mut in_display = false;

            while i < chars.len() {
                if i + 1 < chars.len() && chars[i] == ']' && chars[i + 1] == ']' {
                    i += 2;
                    break;
                }
                if chars[i] == '|' {
                    in_display = true;
                } else if in_display {
                    display.push(chars[i]);
                } else {
                    link.push(chars[i]);
                }
                i += 1;
            }

            if display.is_empty() {
                result.push_str(&link);
            } else {
                result.push_str(&display);
            }
            continue;
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Strip image syntax ![alt](url) -> alt
#[allow(dead_code)]
fn strip_images(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '!' && i + 1 < chars.len() && chars[i + 1] == '[' {
            // Look for ]( pattern
            let mut j = i + 2;
            let mut alt_text = String::new();

            while j < chars.len() && chars[j] != ']' && chars[j] != '\n' {
                alt_text.push(chars[j]);
                j += 1;
            }

            if j < chars.len() && chars[j] == ']' && j + 1 < chars.len() && chars[j + 1] == '(' {
                let mut k = j + 2;
                while k < chars.len() && chars[k] != ')' && chars[k] != '\n' {
                    k += 1;
                }
                if k < chars.len() && chars[k] == ')' {
                    result.push_str(&alt_text);
                    i = k + 1;
                    continue;
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
title: My Note
tags: [rust, programming]
---

# Content here
"#;
        let meta = parse_markdown(content);
        assert_eq!(meta.title, Some("My Note".to_string()));
        assert_eq!(meta.tags, vec!["rust", "programming"]);
    }

    #[test]
    fn test_extract_headings() {
        let content = r#"# Main Title

## Section 1

Some content

### Subsection

## Section 2
"#;
        let meta = parse_markdown(content);
        assert_eq!(meta.headings.len(), 4);
        assert_eq!(meta.headings[0].level, 1);
        assert_eq!(meta.headings[0].text, "Main Title");
    }

    #[test]
    fn test_wiki_links() {
        let content = "Check out [[Other Note]] and [[another|display text]].";
        let meta = parse_markdown(content);
        assert!(meta.links.contains(&"Other Note".to_string()));
        assert!(meta.links.contains(&"another".to_string()));
    }
}
