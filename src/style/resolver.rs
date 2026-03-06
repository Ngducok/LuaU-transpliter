// style/resolver.rs — Resolves CSS rules against the DOM tree.
// Matches CSS selectors to nodes and applies the mapped Luau properties.

use crate::dom::LuauNode;
use crate::parser::css;
use crate::style::mapping;
use anyhow::Result;
use std::collections::HashMap;

/// Tests whether a CSS selector matches a given LuauNode.
/// Supports #id, .class, and tag-name selectors.
fn selector_matches(selector: &str, node: &LuauNode) -> bool {
    let selector = selector.trim();
    if selector.starts_with('#') {
        let id = selector.trim_start_matches('#');
        node.properties.get("id").map(|s| s.as_str()) == Some(id)
            || node.name == id
    } else if selector.starts_with('.') {
        let class = selector.trim_start_matches('.');
        node.properties
            .get("class")
            .map(|s| s.split_whitespace().any(|c| c == class))
            .unwrap_or(false)
    } else {
        let tag = selector.to_lowercase();
        let it = node.instance_type.to_lowercase();
        it == tag
            || (tag == "div" && it == "frame")
            || (tag == "p" && it == "textlabel")
            || (tag == "span" && it == "textlabel")
            || (tag == "button" && it == "textbutton")
            || (tag == "img" && it == "imagelabel")
            || (tag == "imagebutton" && it == "imagebutton")
            || (tag == "input" && (it == "textbox" || it == "textbutton"))
            || (tag == "textarea" && it == "textbox")
            || (tag == "scroll" && it == "scrollingframe")
            || (tag == "canvas" && it == "canvasgroup")
            || (tag == "viewport" && it == "viewportframe")
            || (tag == "video" && it == "videoframe")
            || (tag == "screengui" && it == "screengui")
            || (tag == "billboard" && it == "billboardgui")
            || (tag == "surface" && it == "surfacegui")
    }
}

/// Applies matching CSS rules to a node, merging declarations and mapping to Luau props.
fn resolve_node(node: &mut LuauNode, rules: &[css::CssRule]) {
    let mut merged: HashMap<String, String> = HashMap::new();
    for rule in rules {
        let matches = rule.selectors.iter().any(|s| selector_matches(s, node));
        if matches {
            for (k, v) in &rule.declarations {
                merged.insert(k.clone(), v.clone());
            }
        }
    }
    for (k, v) in &node.properties.clone() {
        if !k.eq_ignore_ascii_case("class") && !k.eq_ignore_ascii_case("id") {
            merged.insert(k.clone(), v.clone());
        }
    }
    let luau_props = mapping::map_css_to_luau(&merged);
    node.properties.clear();
    for (k, v) in luau_props {
        node.properties.insert(k, v);
    }
    for child in &mut node.children {
        resolve_node(child, rules);
    }
}

/// Entry point: parses CSS and resolves all rules against the DOM tree.
pub fn resolve(dom: &mut LuauNode, css_input: &str) -> Result<()> {
    let rules = css::parse(css_input)?;
    resolve_node(dom, &rules);
    Ok(())
}
