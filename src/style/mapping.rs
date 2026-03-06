// style/mapping.rs — Maps CSS properties to Roblox Luau UI properties.
// Handles color conversion, size units (px/%%/vw/vh), layout, and text properties.

use std::collections::HashMap;

/// Converts a CSS width value (px, %, vw) to a UDim2 X-axis representation.
pub fn to_udim2_x(value: &str) -> String {
    let value = value.trim();
    if value.ends_with('%') {
        let val = value
            .trim_end_matches('%')
            .parse::<f32>()
            .unwrap_or(0.0) / 100.0;
        format!("UDim2.fromScale({}, 0)", val)
    } else if value.ends_with("px") {
        let val = value.trim_end_matches("px").parse::<f32>().unwrap_or(0.0);
        format!("UDim2.fromOffset({}, 0)", val)
    } else if value.ends_with("vw") {
        let val = value
            .trim_end_matches("vw")
            .parse::<f32>()
            .unwrap_or(0.0) / 100.0;
        format!("UDim2.new({}, 0, 0, 0)", val)
    } else {
        "UDim2.fromScale(0, 0)".to_string()
    }
}

/// Converts a CSS height value (px, %, vh) to a UDim2 Y-axis representation.
pub fn to_udim2_y(value: &str) -> String {
    let value = value.trim();
    if value.ends_with('%') {
        let val = value
            .trim_end_matches('%')
            .parse::<f32>()
            .unwrap_or(0.0) / 100.0;
        format!("UDim2.fromScale(0, {})", val)
    } else if value.ends_with("px") {
        let val = value.trim_end_matches("px").parse::<f32>().unwrap_or(0.0);
        format!("UDim2.fromOffset(0, {})", val)
    } else if value.ends_with("vh") {
        let val = value
            .trim_end_matches("vh")
            .parse::<f32>()
            .unwrap_or(0.0) / 100.0;
        format!("UDim2.new(0, 0, {}, 0)", val)
    } else {
        "UDim2.fromScale(0, 0)".to_string()
    }
}

pub fn to_udim2_scale(x: f32, y: f32) -> String {
    format!("UDim2.fromScale({}, {})", x, y)
}

pub fn to_udim2_offset(x: f32, y: f32) -> String {
    format!("UDim2.fromOffset({}, {})", x, y)
}

/// Converts a CSS color value (#hex, rgb(), var()) to a Roblox Color3 expression.
pub fn color_to_luau(value: &str) -> String {
    let value = value.trim();
    if value.starts_with('#') {
        let hex = value.trim_start_matches('#');
        let hex = if hex.len() == 3 {
            format!(
                "{}{}{}",
                &hex[0..1].repeat(2),
                &hex[1..2].repeat(2),
                &hex[2..3].repeat(2)
            )
        } else {
            hex.to_string()
        };
        format!("Color3.fromHex(\"{}\")", hex)
    } else if value.starts_with("rgb") {
        let nums: Vec<f32> = value
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == ',' || *c == '.')
            .collect::<String>()
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        if nums.len() >= 3 {
            let r = (nums[0] / 255.0).min(1.0).max(0.0);
            let g = (nums[1] / 255.0).min(1.0).max(0.0);
            let b = (nums[2] / 255.0).min(1.0).max(0.0);
            format!("Color3.new({}, {}, {})", r, g, b)
        } else {
            "Color3.new(0, 0, 0)".to_string()
        }
    } else if value.starts_with("var(") {
        let inner = value
            .trim_start_matches("var(")
            .trim_end_matches(')')
            .trim_start_matches("--")
            .trim();
        let key: String = inner
            .split('-')
            .filter(|s| !s.is_empty())
            .map(|s| {
                let mut c = s.chars();
                match c.next() {
                    Some(f) => format!("{}{}", f.to_uppercase().collect::<String>(), c.as_str()),
                    None => String::new(),
                }
            })
            .collect();
        format!("Theme.{}", key)
    } else {
        "Color3.new(0, 0, 0)".to_string()
    }
}

/// Maps a set of CSS properties to their Roblox Luau UI equivalents.
/// This is the main translation layer between CSS concepts and Roblox properties.
pub fn map_css_to_luau(properties: &HashMap<String, String>) -> HashMap<String, String> {
    let mut luau = HashMap::new();
    for (k, v) in properties {
        let k_lower = k.to_lowercase();
        match k_lower.as_str() {
            "background-color" => {
                luau.insert("BackgroundColor3".to_string(), color_to_luau(v));
            }
            "background" => {
                if v.contains("linear-gradient") || v.contains("gradient") {
                    let s = v.split("gradient").nth(1).unwrap_or("").to_string();
                    let s = s.replace("linear-gradient(", "").replace(")", "");
                    let colors: Vec<&str> = s.split(',').map(|x| x.trim()).filter(|x| x.starts_with('#') || x.starts_with("rgb")).collect();
                    if colors.len() >= 2 {
                        luau.insert("Gradient".to_string(), format!("{} {}", colors[0], colors[1]));
                    } else {
                        luau.insert("BackgroundColor3".to_string(), color_to_luau(v));
                    }
                } else {
                    luau.insert("BackgroundColor3".to_string(), color_to_luau(v));
                }
            }
            "color" => {
                luau.insert("TextColor3".to_string(), color_to_luau(v));
            }
            "display" => {
                if v.trim().eq_ignore_ascii_case("none") {
                    luau.insert("Visible".to_string(), "false".to_string());
                } else if v.trim().eq_ignore_ascii_case("grid") {
                    luau.insert("display".to_string(), "grid".to_string());
                } else if v.trim().eq_ignore_ascii_case("table") {
                    luau.insert("display".to_string(), "table".to_string());
                } else if v.trim().eq_ignore_ascii_case("page") {
                    luau.insert("display".to_string(), "page".to_string());
                } else if v.contains("flex") {
                    luau.insert("display".to_string(), v.clone());
                }
            }
            "width" => {
                luau.insert("SizeX".to_string(), to_udim2_x(v));
            }
            "height" => {
                luau.insert("SizeY".to_string(), to_udim2_y(v));
            }
            "min-height" => {
                if !properties.keys().any(|k| k.eq_ignore_ascii_case("height")) {
                    luau.insert("SizeY".to_string(), to_udim2_y(v));
                }
            }
            "position" => {
                luau.insert("PositionType".to_string(), v.clone());
            }
            "top" => {
                luau.insert("PositionY".to_string(), to_udim2_y(v));
            }
            "left" => {
                luau.insert("PositionX".to_string(), to_udim2_x(v));
            }
            "z-index" => {
                luau.insert("ZIndex".to_string(), v.clone());
            }
            "padding" => {
                luau.insert("Padding".to_string(), v.clone());
            }
            "padding-left" => {
                luau.insert("PaddingLeft".to_string(), v.clone());
            }
            "padding-right" => {
                luau.insert("PaddingRight".to_string(), v.clone());
            }
            "padding-top" => {
                luau.insert("PaddingTop".to_string(), v.clone());
            }
            "padding-bottom" => {
                luau.insert("PaddingBottom".to_string(), v.clone());
            }
            "border-radius" => {
                luau.insert("CornerRadius".to_string(), v.clone());
            }
            "flex-direction" => {
                luau.insert("FlexDirection".to_string(), v.clone());
            }
            "flex-grow" => {
                luau.insert("FlexGrow".to_string(), v.clone());
            }
            "justify-content" => {
                luau.insert("JustifyContent".to_string(), v.clone());
            }
            "align-items" => {
                luau.insert("AlignItems".to_string(), v.clone());
            }
            "gap" | "row-gap" => {
                let val = v.trim_end_matches("px").parse::<f32>().unwrap_or(0.0);
                luau.insert("LayoutGap".to_string(), format!("{}", val as i32));
            }
            "opacity" | "background-transparency" => {
                let val = v.trim().parse::<f32>().unwrap_or(1.0);
                luau.insert("BackgroundTransparency".to_string(), format!("{}", 1.0 - val));
            }
            "border" | "border-width" => {
                let val = v.trim_end_matches("px").parse::<f32>().unwrap_or(0.0);
                luau.insert("BorderSizePixel".to_string(), format!("{}", val as i32));
            }
            "border-color" => {
                luau.insert("BorderColor3".to_string(), color_to_luau(v));
            }
            "overflow" => {
                if v.trim().eq_ignore_ascii_case("hidden") {
                    luau.insert("ClipsDescendants".to_string(), "true".to_string());
                }
            }
            "transform" | "rotate" => {
                let deg = v
                    .replace("deg", "")
                    .replace("rotate(", "")
                    .replace(")", "")
                    .trim()
                    .parse::<f32>()
                    .unwrap_or(0.0);
                luau.insert("Rotation".to_string(), format!("{}", deg));
            }
            "transform-origin" | "anchor-point" => {
                let parts: Vec<f32> = v
                    .split_whitespace()
                    .filter_map(|s| s.trim_end_matches('%').parse().ok())
                    .collect();
                let x = parts.get(0).copied().unwrap_or(0.5) / 100.0;
                let y = parts.get(1).copied().unwrap_or(0.5) / 100.0;
                luau.insert("AnchorPoint".to_string(), format!("Vector2.new({}, {})", x, y));
            }
            "transition" => {
                luau.insert("Transition".to_string(), v.clone());
            }
            "animation" => {
                luau.insert("Animation".to_string(), v.clone());
            }
            "layout-order" => {
                luau.insert("LayoutOrder".to_string(), v.clone());
            }
            "automatic-size" => {
                let v_lower = v.to_lowercase();
                if v_lower.contains("x") && v_lower.contains("y") {
                    luau.insert("AutomaticSize".to_string(), "Enum.AutomaticSize.XY".to_string());
                } else if v_lower.contains("x") {
                    luau.insert("AutomaticSize".to_string(), "Enum.AutomaticSize.X".to_string());
                } else if v_lower.contains("y") {
                    luau.insert("AutomaticSize".to_string(), "Enum.AutomaticSize.Y".to_string());
                }
            }
            "draggable" => {
                if v.trim().eq_ignore_ascii_case("true") {
                    luau.insert("Draggable".to_string(), "true".to_string());
                }
            }
            "active" => {
                if v.trim().eq_ignore_ascii_case("true") {
                    luau.insert("Active".to_string(), "true".to_string());
                }
            }
            "selectable" => {
                if v.trim().eq_ignore_ascii_case("true") {
                    luau.insert("Selectable".to_string(), "true".to_string());
                }
            }
            "text-size" | "font-size" => {
                let val = v.trim_end_matches("px").parse::<f32>().unwrap_or(14.0);
                luau.insert("TextSize".to_string(), format!("{}", val as i32));
            }
            "font" | "font-family" => {
                luau.insert("Font".to_string(), format!("Enum.Font.{}", v.trim().replace(' ', "")));
            }
            "text-wrap" => {
                if v.trim().eq_ignore_ascii_case("true") {
                    luau.insert("TextWrapped".to_string(), "true".to_string());
                }
            }
            "text-x-align" | "text-align" => {
                let a = v.to_lowercase();
                let align = if a.contains("center") {
                    "Enum.TextXAlignment.Center"
                } else if a.contains("right") {
                    "Enum.TextXAlignment.Right"
                } else {
                    "Enum.TextXAlignment.Left"
                };
                luau.insert("TextXAlignment".to_string(), align.to_string());
            }
            "text-y-align" | "vertical-align" => {
                let a = v.to_lowercase();
                let align = if a.contains("center") {
                    "Enum.TextYAlignment.Center"
                } else if a.contains("bottom") {
                    "Enum.TextYAlignment.Bottom"
                } else {
                    "Enum.TextYAlignment.Top"
                };
                luau.insert("TextYAlignment".to_string(), align.to_string());
            }
            "image-color" => {
                luau.insert("ImageColor3".to_string(), color_to_luau(v));
            }
            "image-transparency" => {
                let val = v.trim().parse::<f32>().unwrap_or(0.0);
                luau.insert("ImageTransparency".to_string(), format!("{}", val));
            }
            "scale-type" | "object-fit" => {
                let v_lower = v.to_lowercase();
                let scale = if v_lower.contains("fill") {
                    "Enum.ScaleType.Stretch"
                } else if v_lower.contains("contain") {
                    "Enum.ScaleType.Fit"
                } else if v_lower.contains("cover") {
                    "Enum.ScaleType.Crop"
                } else {
                    "Enum.ScaleType.Stretch"
                };
                luau.insert("ScaleType".to_string(), scale.to_string());
            }
            "scroll-bar-thickness" => {
                let val = v.trim_end_matches("px").parse::<f32>().unwrap_or(0.0);
                luau.insert("ScrollBarThickness".to_string(), format!("{}", val as i32));
            }
            "canvas-size" => {
                luau.insert("CanvasSize".to_string(), v.clone());
            }
            "aspect-ratio" => {
                luau.insert("AspectRatio".to_string(), v.clone());
            }
            "grid-cell-width" => {
                luau.insert("GridCellWidth".to_string(), v.clone());
            }
            "grid-cell-height" => {
                luau.insert("GridCellHeight".to_string(), v.clone());
            }
            "grid-padding" => {
                luau.insert("GridPadding".to_string(), v.clone());
            }
            "gradient" => {
                luau.insert("Gradient".to_string(), v.clone());
            }
            "ui-stroke" | "stroke" => {
                luau.insert("UIStroke".to_string(), v.clone());
            }
            "ui-scale" | "scale" => {
                luau.insert("UIScale".to_string(), v.clone());
            }
            _ => {
                luau.insert(k.clone(), v.clone());
            }
        }
    }
    luau
}
