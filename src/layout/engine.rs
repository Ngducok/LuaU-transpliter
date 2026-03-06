use crate::dom::LuauNode;

fn parse_padding(value: &str) -> (f32, f32, f32, f32) {
    let parts: Vec<f32> = value
        .split_whitespace()
        .filter_map(|s| s.trim_end_matches("px").parse().ok())
        .collect();
    match parts.len() {
        1 => (parts[0], parts[0], parts[0], parts[0]),
        2 => (parts[0], parts[1], parts[0], parts[1]),
        4 => (parts[0], parts[1], parts[2], parts[3]),
        _ => (0.0, 0.0, 0.0, 0.0),
    }
}

fn parse_corner_radius(value: &str) -> String {
    let value = value.trim();
    if value.ends_with('%') {
        let val = value
            .trim_end_matches('%')
            .parse::<f32>()
            .unwrap_or(0.0);
        if (val - 50.0).abs() < 1.0 {
            "UDim.new(0.5, 0)".to_string()
        } else {
            format!("UDim.new({}, 0)", val / 100.0)
        }
    } else if value.ends_with("px") {
        let val = value.trim_end_matches("px").parse::<f32>().unwrap_or(0.0);
        format!("UDim.new(0, {})", val)
    } else {
        "UDim.new(0, 0)".to_string()
    }
}

fn parse_padding_value(v: &str) -> f32 {
    v.trim_end_matches("px").parse::<f32>().unwrap_or(0.0)
}

fn inject_helpers(node: &mut LuauNode, parent_flex_column: Option<bool>) {
    if let Some(is_column) = parent_flex_column {
        let has_size = node.properties.contains_key("SizeX") || node.properties.contains_key("SizeY");
        if !has_size {
            let auto = if is_column {
                "Enum.AutomaticSize.Y"
            } else {
                "Enum.AutomaticSize.X"
            };
            node.properties.insert("AutomaticSize".to_string(), auto.to_string());
        }
    }
    let mut need_padding = false;
    let (mut left, mut right, mut top, mut bottom) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
    if let Some(padding) = node.properties.get("Padding") {
        (left, right, top, bottom) = parse_padding(padding);
        need_padding = true;
    }
    if let Some(pl) = node.properties.get("PaddingLeft") {
        left = parse_padding_value(pl);
        need_padding = true;
    }
    if let Some(pr) = node.properties.get("PaddingRight") {
        right = parse_padding_value(pr);
        need_padding = true;
    }
    if let Some(pt) = node.properties.get("PaddingTop") {
        top = parse_padding_value(pt);
        need_padding = true;
    }
    if let Some(pb) = node.properties.get("PaddingBottom") {
        bottom = parse_padding_value(pb);
        need_padding = true;
    }
    if need_padding {
        let mut ui_padding = LuauNode::new("UIPadding", "UIPadding");
        ui_padding.set_property("PaddingLeft", &format!("UDim.new(0, {})", left));
        ui_padding.set_property("PaddingRight", &format!("UDim.new(0, {})", right));
        ui_padding.set_property("PaddingTop", &format!("UDim.new(0, {})", top));
        ui_padding.set_property("PaddingBottom", &format!("UDim.new(0, {})", bottom));
        node.helpers.insert(0, ui_padding);
    }
    node.properties.remove("Padding");
    node.properties.remove("PaddingLeft");
    node.properties.remove("PaddingRight");
    node.properties.remove("PaddingTop");
    node.properties.remove("PaddingBottom");
    if let Some(cr) = node.properties.get("CornerRadius") {
        let mut ui_corner = LuauNode::new("UICorner", "UICorner");
        ui_corner.set_property("CornerRadius", &parse_corner_radius(cr));
        node.helpers.insert(0, ui_corner);
        node.properties.remove("CornerRadius");
    }
    let has_flex = node
        .properties
        .get("FlexDirection")
        .or_else(|| node.properties.get("display"))
        .map(|v| v.contains("flex") || v.eq_ignore_ascii_case("row") || v.eq_ignore_ascii_case("column"))
        .unwrap_or(false);
    let flex_column = has_flex.then(|| {
        let direction = node
            .properties
            .get("FlexDirection")
            .map(|s| s.as_str())
            .unwrap_or("row");
        direction.eq_ignore_ascii_case("column")
    });
    if has_flex {
        let mut ui_list = LuauNode::new("UIListLayout", "UIListLayout");
        let is_column = flex_column.unwrap_or(false);
        if is_column {
            ui_list.set_property("FillDirection", "Enum.FillDirection.Vertical");
        } else {
            ui_list.set_property("FillDirection", "Enum.FillDirection.Horizontal");
        }
        let justify = node.properties.get("JustifyContent").map(|s| s.as_str()).unwrap_or("flex-start");
        let align = node.properties.get("AlignItems").map(|s| s.as_str()).unwrap_or("stretch");
        let vert = |v: &str| -> String {
            match v.to_lowercase().as_str() {
                "center" => "Enum.VerticalAlignment.Center".to_string(),
                "flex-end" | "end" => "Enum.VerticalAlignment.Bottom".to_string(),
                _ => "Enum.VerticalAlignment.Top".to_string(),
            }
        };
        let horiz = |v: &str| -> String {
            match v.to_lowercase().as_str() {
                "center" => "Enum.HorizontalAlignment.Center".to_string(),
                "flex-end" | "end" => "Enum.HorizontalAlignment.Right".to_string(),
                _ => "Enum.HorizontalAlignment.Left".to_string(),
            }
        };
        if is_column {
            ui_list.set_property("VerticalAlignment", &vert(justify));
            ui_list.set_property("HorizontalAlignment", &horiz(align));
        } else {
            ui_list.set_property("HorizontalAlignment", &horiz(justify));
            ui_list.set_property("VerticalAlignment", &vert(align));
        }
        if let Some(gap) = node.properties.get("LayoutGap") {
            if let Ok(n) = gap.parse::<i32>() {
                ui_list.set_property("Padding", &format!("UDim.new(0, {})", n));
            }
            node.properties.remove("LayoutGap");
        }
        node.helpers.insert(0, ui_list);
        node.properties.remove("FlexDirection");
        node.properties.remove("display");
        node.properties.remove("JustifyContent");
        node.properties.remove("AlignItems");
    }
    if let Some(ar) = node.properties.get("AspectRatio") {
        let mut aspect = LuauNode::new("UIAspectRatioConstraint", "UIAspectRatioConstraint");
        aspect.set_property("AspectRatio", ar);
        node.helpers.insert(0, aspect);
        node.properties.remove("AspectRatio");
    }
    let display = node.properties.get("Display").or_else(|| node.properties.get("display")).cloned();
    if display.as_deref().map(|v| v.eq_ignore_ascii_case("grid")).unwrap_or(false) {
        let mut grid = LuauNode::new("UIGridLayout", "UIGridLayout");
        if let Some(cw) = node.properties.get("GridCellWidth") {
            grid.set_property("CellSize", &format!("UDim2.new(0, {}, 0, {})",
                cw.trim_end_matches("px").parse::<f32>().unwrap_or(100.0),
                node.properties.get("GridCellHeight").map(|h| h.trim_end_matches("px").parse::<f32>().unwrap_or(100.0)).unwrap_or(100.0)));
            node.properties.remove("GridCellWidth");
            node.properties.remove("GridCellHeight");
        }
        if let Some(pad) = node.properties.get("GridPadding") {
            let v: f32 = pad.trim_end_matches("px").parse().unwrap_or(0.0);
            grid.set_property("CellPadding", &format!("UDim2.new(0, {}, 0, {})", v, v));
            node.properties.remove("GridPadding");
        }
        node.helpers.insert(0, grid);
        node.properties.remove("Display");
        node.properties.remove("display");
    }
    if display.as_deref().map(|v| v.eq_ignore_ascii_case("table")).unwrap_or(false) {
        let tbl = LuauNode::new("UITableLayout", "UITableLayout");
        node.helpers.insert(0, tbl);
        node.properties.remove("Display");
        node.properties.remove("display");
    }
    if display.as_deref().map(|v| v.eq_ignore_ascii_case("page")).unwrap_or(false) {
        let page = LuauNode::new("UIPageLayout", "UIPageLayout");
        node.helpers.insert(0, page);
        node.properties.remove("Display");
        node.properties.remove("display");
    }
    if let Some(grad) = node.properties.get("Gradient") {
        let mut ui_grad = LuauNode::new("UIGradient", "UIGradient");
        let parts: Vec<&str> = grad.split_whitespace().collect();
        if parts.len() >= 2 {
            let c1 = crate::style::mapping::color_to_luau(parts[0]);
            let c2 = crate::style::mapping::color_to_luau(parts[1]);
            ui_grad.set_property("Color", &format!("ColorSequence.new({{ColorSequenceKeypoint.new(0, {}), ColorSequenceKeypoint.new(1, {})}})", c1, c2));
        }
        if parts.len() >= 3 {
            let rot: f32 = parts[2].parse().unwrap_or(0.0);
            ui_grad.set_property("Rotation", &format!("{}", rot));
        }
        node.helpers.insert(0, ui_grad);
        node.properties.remove("Gradient");
    }
    if let Some(stroke) = node.properties.get("UIStroke") {
        let mut ui_stroke = LuauNode::new("UIStroke", "UIStroke");
        let parts: Vec<&str> = stroke.split_whitespace().collect();
        if let Some(thick) = parts.first() {
            ui_stroke.set_property("Thickness", &format!("{}", thick.trim_end_matches("px").parse::<f32>().unwrap_or(1.0)));
        }
        if parts.len() > 1 {
            ui_stroke.set_property("Color", &crate::style::mapping::color_to_luau(parts[1]));
        }
        node.helpers.insert(0, ui_stroke);
        node.properties.remove("UIStroke");
    }
    if let Some(scale) = node.properties.get("UIScale") {
        let mut ui_scale = LuauNode::new("UIScale", "UIScale");
        ui_scale.set_property("Scale", scale);
        node.helpers.insert(0, ui_scale);
        node.properties.remove("UIScale");
    }
    for (idx, child) in node.children.iter_mut().enumerate() {
        if let Some(fg) = child.properties.get("FlexGrow") {
            if fg == "1" {
                let mut ui_flex = LuauNode::new("UIFlexItem", "UIFlexItem");
                ui_flex.set_property("FlexMode", "Fill");
                child.helpers.insert(0, ui_flex);
            }
            child.properties.remove("FlexGrow");
        }
        if let Some(is_col) = flex_column {
            let has_size = child.properties.contains_key("SizeX") || child.properties.contains_key("SizeY");
            if !has_size {
                let it = child.instance_type.to_lowercase();
                if matches!(it.as_str(), "textlabel" | "textbutton" | "textbox" | "frame" | "imagebutton" | "imagelabel") {
                    let as_val = if is_col { "Enum.AutomaticSize.Y" } else { "Enum.AutomaticSize.X" };
                    child.properties.insert("AutomaticSize".to_string(), as_val.to_string());
                }
            }
            // Ensure correct layout order - UIListLayout sorts by LayoutOrder
            child.properties.insert("LayoutOrder".to_string(), (idx + 1).to_string());
        }
        inject_helpers(child, flex_column);
    }
}

fn parse_udim2_x(s: &str) -> (f32, f32) {
    let s = s.trim();
    if s.starts_with("UDim2.fromScale(") {
        let inner = s.trim_start_matches("UDim2.fromScale(").trim_end_matches(")");
        let parts: Vec<f32> = inner.split(',').filter_map(|p| p.trim().parse().ok()).collect();
        (parts.get(0).copied().unwrap_or(0.0), 0.0)
    } else if s.starts_with("UDim2.fromOffset(") {
        let inner = s.trim_start_matches("UDim2.fromOffset(").trim_end_matches(")");
        let parts: Vec<f32> = inner.split(',').filter_map(|p| p.trim().parse().ok()).collect();
        (0.0, parts.get(0).copied().unwrap_or(0.0))
    } else if s.starts_with("UDim2.new(") {
        let inner = s.trim_start_matches("UDim2.new(").trim_end_matches(")");
        let parts: Vec<f32> = inner.split(',').filter_map(|p| p.trim().parse().ok()).collect();
        (
            parts.get(0).copied().unwrap_or(0.0),
            parts.get(1).copied().unwrap_or(0.0),
        )
    } else {
        (0.0, 0.0)
    }
}

fn parse_udim2_y(s: &str) -> (f32, f32) {
    let s = s.trim();
    if s.starts_with("UDim2.fromScale(") {
        let inner = s.trim_start_matches("UDim2.fromScale(").trim_end_matches(")");
        let parts: Vec<f32> = inner.split(',').filter_map(|p| p.trim().parse().ok()).collect();
        (parts.get(1).copied().unwrap_or(0.0), 0.0)
    } else if s.starts_with("UDim2.fromOffset(") {
        let inner = s.trim_start_matches("UDim2.fromOffset(").trim_end_matches(")");
        let parts: Vec<f32> = inner.split(',').filter_map(|p| p.trim().parse().ok()).collect();
        (0.0, parts.get(1).copied().unwrap_or(0.0))
    } else if s.starts_with("UDim2.new(") {
        let inner = s.trim_start_matches("UDim2.new(").trim_end_matches(")");
        let parts: Vec<f32> = inner.split(',').filter_map(|p| p.trim().parse().ok()).collect();
        (
            parts.get(2).copied().unwrap_or(0.0),
            parts.get(3).copied().unwrap_or(0.0),
        )
    } else {
        (0.0, 0.0)
    }
}

fn build_size(node: &LuauNode) -> Option<String> {
    let x = node.properties.get("SizeX");
    let y = node.properties.get("SizeY");
    match (x, y) {
        (Some(a), Some(b)) => {
            let (sx, ox) = parse_udim2_x(a);
            let (sy, oy) = parse_udim2_y(b);
            Some(format!("UDim2.new({}, {}, {}, {})", sx, ox, sy, oy))
        }
        (Some(a), None) => {
            let (sx, ox) = parse_udim2_x(a);
            Some(format!("UDim2.new({}, {}, 1, 0)", sx, ox))
        }
        (None, Some(b)) => {
            let (sy, oy) = parse_udim2_y(b);
            Some(format!("UDim2.new(0, 0, {}, {})", sy, oy))
        }
        _ => None,
    }
}

fn finalize_properties(node: &mut LuauNode, is_root: bool) {
    if let Some(size) = build_size(node) {
        node.properties.insert("Size".to_string(), size);
        node.properties.remove("SizeX");
        node.properties.remove("SizeY");
    } else if is_root {
        node.properties.insert("Size".to_string(), "UDim2.new(1, 0, 1, 0)".to_string());
    }
}

fn transform_node(node: &mut LuauNode, is_root: bool) {
    inject_helpers(node, None);
    finalize_properties(node, is_root);
    for child in &mut node.children {
        transform_node(child, false);
    }
}

pub fn transform(mut node: LuauNode) -> LuauNode {
    transform_node(&mut node, true);
    node
}
