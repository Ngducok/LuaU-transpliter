// parser/css.rs — Simple CSS parser that extracts selector-declaration rule blocks.
// Handles nested @-rules (media queries, keyframes) by skipping them.

use anyhow::Result;
use std::collections::HashMap;

/// Represents a single CSS rule: one or more selectors and their key-value declarations.
pub struct CssRule {
    pub selectors: Vec<String>,
    pub declarations: HashMap<String, String>,
}

/// Parses CSS source into a list of CssRule blocks.
/// Skips @-rules (media queries, keyframes, etc.) entirely.
pub fn parse(css: &str) -> Result<Vec<CssRule>> {
    if css.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut rules = Vec::new();
    let mut i = 0;
    let bytes = css.as_bytes();
    while i < bytes.len() {
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\n' || bytes[i] == b'\r' || bytes[i] == b'\t') {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        if bytes[i] == b'@' {
            while i < bytes.len() && bytes[i] != b'{' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
                let mut depth = 1;
                while i < bytes.len() && depth > 0 {
                    if bytes[i] == b'{' {
                        depth += 1;
                    } else if bytes[i] == b'}' {
                        depth -= 1;
                    }
                    i += 1;
                }
            }
            continue;
        }
        let sel_start = i;
        while i < bytes.len() && bytes[i] != b'{' {
            i += 1;
        }
        let selectors_str = std::str::from_utf8(&bytes[sel_start..i]).unwrap_or("").trim();
        if selectors_str.is_empty() {
            continue;
        }
        let selectors: Vec<String> = selectors_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if i < bytes.len() {
            i += 1;
        }
        let decl_start = i;
        let mut depth = 1;
        while i < bytes.len() && depth > 0 {
            if bytes[i] == b'{' {
                depth += 1;
            } else if bytes[i] == b'}' {
                depth -= 1;
            }
            i += 1;
        }
        let decl_str = if depth == 0 {
            std::str::from_utf8(&bytes[decl_start..i.saturating_sub(1)]).unwrap_or("")
        } else {
            ""
        };
        let mut declarations = HashMap::new();
        for part in decl_str.split(';') {
            let part = part.trim();
            if let Some((k, v)) = part.split_once(':') {
                let key = k.trim().to_string();
                let val = v.trim().to_string();
                if !key.is_empty() {
                    declarations.insert(key, val);
                }
            }
        }
        if !selectors.is_empty() {
            rules.push(CssRule {
                selectors,
                declarations,
            });
        }
    }
    Ok(rules)
}
