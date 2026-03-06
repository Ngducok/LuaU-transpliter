use crate::dom::LuauNode;
use anyhow::Result;
use std::cell::RefCell;
use kuchiki::traits::TendrilSink;

fn instance_type_from_tag(tag: &str, input_type: Option<&str>) -> &'static str {
    match tag.to_lowercase().as_str() {
        "screengui" | "gui" => "ScreenGui",
        "billboard" | "billboardgui" => "BillboardGui",
        "surface" | "surfacegui" => "SurfaceGui",
        "div" | "section" | "main" | "header" | "footer" | "article" | "nav" | "aside" => "Frame",
        "p" | "span" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "label" | "figcaption" | "caption" => "TextLabel",
        "button" | "a" => "TextButton",
        "img" | "picture" => "ImageLabel",
        "input" => match input_type {
            Some(s) if s.eq_ignore_ascii_case("checkbox") || s.eq_ignore_ascii_case("radio") => "TextButton",
            _ => "TextBox",
        },
        "scroll" | "scrollview" => "ScrollingFrame",
        "textarea" => "TextBox",
        "canvas" => "CanvasGroup",
        "viewport" => "ViewportFrame",
        "video" => "VideoFrame",
        "progress" | "meter" => "Frame",
        "hr" => "Frame",
        "imagebutton" | "imgbutton" => "ImageButton",
        _ => "Frame",
    }
}

fn is_text_container(tag: &str) -> bool {
    matches!(
        tag.to_lowercase().as_str(),
        "p" | "span" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "label" | "button" | "a"
    )
}

fn is_inline_formatting(tag: &str) -> bool {
    matches!(tag.to_lowercase().as_str(), "b" | "i" | "u" | "span")
}

fn parse_inline_style(style: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for part in style.split(';') {
        let part = part.trim();
        if let Some((k, v)) = part.split_once(':') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    map
}

fn color_from_style(style: &std::collections::HashMap<String, String>) -> Option<String> {
    style.get("color").cloned().or_else(|| {
        style.get("background-color").cloned()
    })
}

fn css_color_to_hex(color: &str) -> String {
    let color = color.trim();
    if color.starts_with('#') {
        let hex = color.trim_start_matches('#');
        if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0);
            format!("#{:02x}{:02x}{:02x}", r, g, b)
        } else {
            format!("#{}", hex)
        }
    } else if color.starts_with("rgb") {
        let nums: Vec<u8> = color
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == ',')
            .collect::<String>()
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        if nums.len() >= 3 {
            format!("#{:02x}{:02x}{:02x}", nums[0], nums[1], nums[2])
        } else {
            "#000000".to_string()
        }
    } else {
        "#000000".to_string()
    }
}

fn collect_rich_text(node: &kuchiki::NodeRef) -> String {
    let mut out = String::new();
    for child in node.children() {
        if let Some(text) = child.as_text() {
            out.push_str(&text.borrow().clone());
        } else if let Some(el) = child.as_element() {
            let tag = el.name.local.as_ref();
            let inner: String = collect_rich_text(&child);
            if tag.eq_ignore_ascii_case("b") {
                out.push_str(&format!("<b>{}</b>", inner));
            } else if tag.eq_ignore_ascii_case("i") {
                out.push_str(&format!("<i>{}</i>", inner));
            } else if tag.eq_ignore_ascii_case("u") {
                out.push_str(&format!("<u>{}</u>", inner));
            } else if tag.eq_ignore_ascii_case("span") {
                let attrs = el.attributes.borrow();
                let style = attrs.get("style").map(|s| s.to_string());
                if let Some(style_str) = style {
                    let style_map = parse_inline_style(&style_str);
                    if let Some(color) = color_from_style(&style_map) {
                        let hex = css_color_to_hex(&color);
                        out.push_str(&format!(r#"<font color="{}">{}</font>"#, hex, inner));
                    } else {
                        out.push_str(&inner);
                    }
                } else {
                    out.push_str(&inner);
                }
            } else {
                out.push_str(&inner);
            }
        }
    }
    out
}

fn traverse(node: &kuchiki::NodeRef, counter: &RefCell<u32>) -> Option<LuauNode> {
    let el = node.as_element()?;
    let tag = el.name.local.as_ref().to_string();
    let mut counter_mut = counter.borrow_mut();
    *counter_mut += 1;
    let id = *counter_mut;
    drop(counter_mut);

    let input_type_str = el.attributes.borrow().get("type").map(|s| s.to_string());
    let instance_type = instance_type_from_tag(&tag, input_type_str.as_deref()).to_string();
    let name = el
        .attributes
        .borrow()
        .get("id")
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}_{}", tag, id));

    let mut luau = LuauNode::new(&instance_type, &name);

    if let Some(style_attr) = el.attributes.borrow().get("style") {
        let style_map = parse_inline_style(style_attr);
        for (k, v) in style_map {
            luau.set_property(k, v);
        }
    }

    if let Some(class) = el.attributes.borrow().get("class") {
        luau.set_property("class", class);
    }
    if let Some(id) = el.attributes.borrow().get("id") {
        luau.set_property("id", id);
    }

    if is_text_container(&tag) {
        let rich = collect_rich_text(node);
        if !rich.is_empty() {
            luau.set_property("Text", &rich);
            luau.set_property("RichText", "true");
        }
    }

    if tag.eq_ignore_ascii_case("img") || tag.eq_ignore_ascii_case("imagebutton") || tag.eq_ignore_ascii_case("imgbutton") {
        if let Some(src) = el.attributes.borrow().get("src") {
            luau.set_property("Image", src);
        }
    }

    if tag.eq_ignore_ascii_case("button") || tag.eq_ignore_ascii_case("a") || tag.eq_ignore_ascii_case("imagebutton") {
        if let Some(onclick) = el.attributes.borrow().get("data-onclick") {
            luau.set_property("data-onclick", onclick);
        }
    }
    if let Some(transition) = el.attributes.borrow().get("data-transition") {
        luau.set_property("data-transition", transition);
    }
    if let Some(animate) = el.attributes.borrow().get("data-animate") {
        luau.set_property("data-animate", animate);
    }
    if let Some(animate_on) = el.attributes.borrow().get("data-animate-on") {
        luau.set_property("data-animate-on", animate_on);
    }
    if tag.eq_ignore_ascii_case("textarea") {
        luau.set_property("MultiLine", "true");
    }
    if tag.eq_ignore_ascii_case("input") {
        if let Some(input_type) = el.attributes.borrow().get("type") {
            if input_type.eq_ignore_ascii_case("password") {
                luau.set_property("ClearTextOnFocus", "false");
            }
        }
        if let Some(placeholder) = el.attributes.borrow().get("placeholder") {
            luau.set_property("PlaceholderText", placeholder);
        }
    }

    for child in node.children() {
        if let Some(el) = child.as_element() {
            let child_tag = el.name.local.as_ref();
            if is_text_container(&tag) && is_inline_formatting(child_tag) {
                continue;
            }
        } else if child.as_text().is_some() {
            continue;
        }
        if let Some(child_luau) = traverse(&child, counter) {
            luau.add_child(child_luau);
        }
    }

    Some(luau)
}

pub fn parse(html: &str) -> Result<Option<LuauNode>> {
    let document = kuchiki::parse_html().one(html);
    let counter = RefCell::new(0u32);

    let root = document
        .select_first("body")
        .ok()
        .or_else(|| document.select_first("html").ok())
        .or_else(|| document.select_first("div").ok())
        .or_else(|| {
            document
                .select_first("*")
                .ok()
        });

    let root: kuchiki::NodeDataRef<kuchiki::ElementData> = match root {
        Some(r) => r,
        None => return Ok(None),
    };

    let root_node = root.as_node().clone();
    let result = traverse(&root_node, &counter);
    Ok(result)
}
