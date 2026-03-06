use crate::dom::LuauNode;
use anyhow::Result;

fn indent(level: u32) -> String {
    "    ".repeat(level as usize)
}

fn escape_lua_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn emit_node(node: &LuauNode, parent_var: &str, var_counter: &mut u32, out: &mut String, level: u32) {
    *var_counter += 1;
    let var_name = format!("{}_{}", node.instance_type.to_lowercase(), var_counter);
    let ind = indent(level);

    out.push_str(&format!(
        "{}local {} = Instance.new({})\n",
        ind,
        var_name,
        escape_lua_string(&node.instance_type)
    ));
    out.push_str(&format!("{}{}.Parent = {}\n", ind, var_name, parent_var));
    out.push_str(&format!("{}{}.Name = {}\n", ind, var_name, escape_lua_string(&node.name)));

    // Emit Size and key properties before helpers so layout calculates correctly
    if let Some(size) = node.properties.get("Size") {
        let luau_val = map_property_to_luau("Size", size);
        out.push_str(&format!("{}{}.Size = {}\n", ind, var_name, luau_val));
    }
    if let Some(bg) = node.properties.get("BackgroundColor3") {
        let luau_val = map_property_to_luau("BackgroundColor3", bg);
        out.push_str(&format!("{}{}.BackgroundColor3 = {}\n", ind, var_name, luau_val));
    }
    if let Some(bs) = node.properties.get("BorderSizePixel") {
        let luau_val = map_property_to_luau("BorderSizePixel", bs);
        out.push_str(&format!("{}{}.BorderSizePixel = {}\n", ind, var_name, luau_val));
    }

    for helper in &node.helpers {
        *var_counter += 1;
        let h_var = format!("{}_{}", helper.instance_type.to_lowercase(), var_counter);
        out.push_str(&format!(
            "{}local {} = Instance.new({})\n",
            ind,
            h_var,
            escape_lua_string(&helper.instance_type)
        ));
        out.push_str(&format!("{}{}.Parent = {}\n", ind, h_var, var_name));
        for (k, v) in &helper.properties {
            if k.contains('-') {
                continue;
            }
            let luau_val = map_property_to_luau(k, v);
            out.push_str(&format!("{}{}.{} = {}\n", ind, h_var, k, luau_val));
        }
    }

    for (k, v) in &node.properties {
        if k == "class" || k.starts_with("data-") || k.contains('-')
            || k.eq_ignore_ascii_case("JustifyContent") || k.eq_ignore_ascii_case("AlignItems")
            || k.eq_ignore_ascii_case("FlexGrow")
            || k.eq_ignore_ascii_case("Size") || k.eq_ignore_ascii_case("BackgroundColor3")
            || k.eq_ignore_ascii_case("BorderSizePixel")
        {
            continue;
        }
        let luau_val = map_property_to_luau(k, v);
        out.push_str(&format!("{}{}.{} = {}\n", ind, var_name, k, luau_val));
    }

    if node.properties.contains_key("data-onclick") {
        if let Some(handler) = node.properties.get("data-onclick") {
            out.push_str(&format!(
                "{}{}.Activated:Connect(function()\n",
                ind, var_name
            ));
            out.push_str(&format!(
                "{}    if Controller[{}] then\n",
                indent(level + 1),
                escape_lua_string(handler)
            ));
            out.push_str(&format!(
                "{}        Controller[{}]()\n",
                indent(level + 1),
                escape_lua_string(handler)
            ));
            out.push_str(&format!("{}    end\n", indent(level + 1)));
            out.push_str(&format!("{}end)\n", ind));
        }
    }
    if let Some(transition) = node.properties.get("data-transition") {
        emit_transition(&var_name, transition, &ind, level, out);
    }
    if let Some(transition) = node.properties.get("Transition") {
        emit_transition(&var_name, transition, &ind, level, out);
    }
    if let Some(animate) = node.properties.get("data-animate") {
        emit_animate_on_mount(&var_name, animate, &ind, level, out);
    }
    if let Some(animate) = node.properties.get("Animation") {
        emit_animate_on_mount(&var_name, animate, &ind, level, out);
    }

    for child in &node.children {
        emit_node(child, &var_name, var_counter, out, level + 1);
    }
}

fn map_property_to_luau(key: &str, value: &str) -> String {
    if value.starts_with("Color3.") || value.starts_with("UDim2.") || value.starts_with("UDim.")
        || value.starts_with("Vector2.") || value.starts_with("Enum.") || value == "true" || value == "false"
    {
        return value.to_string();
    }
    let key_lower = key.to_lowercase();
    if matches!(key_lower.as_str(), "bordersizepixel" | "textsize" | "scrollbarthickness" | "layoutorder") {
        if let Ok(n) = value.trim().parse::<i32>() {
            return n.to_string();
        }
    }
    if key_lower == "imagetransparency" || key_lower == "backgroundtransparency" {
        if let Ok(n) = value.trim().parse::<f32>() {
            return n.to_string();
        }
    }
    match key_lower.as_str() {
        "richtext" => value.to_string(),
        "text" => escape_lua_string(value),
        "image" => escape_lua_string(value),
        "name" => escape_lua_string(value),
        "placeholdertext" => escape_lua_string(value),
        _ => escape_lua_string(value),
    }
}

fn parse_easing(easing: &str) -> (String, String) {
    let e = easing.to_lowercase();
    let style = if e.contains("elastic") {
        "Elastic"
    } else if e.contains("back") {
        "Back"
    } else if e.contains("bounce") {
        "Bounce"
    } else if e.contains("expo") || e.contains("exponential") {
        "Exponential"
    } else if e.contains("quad") {
        "Quad"
    } else if e.contains("cubic") {
        "Cubic"
    } else if e.contains("quart") {
        "Quart"
    } else if e.contains("quint") {
        "Quint"
    } else if e.contains("sine") {
        "Sine"
    } else if e.contains("circ") || e.contains("circle") {
        "Circ"
    } else {
        "Quad"
    };
    let dir = if e.contains("inout") {
        "InOut"
    } else if e.contains("in") && !e.contains("out") {
        "In"
    } else {
        "Out"
    };
    (format!("Enum.EasingStyle.{}", style), format!("Enum.EasingDirection.{}", dir))
}

fn emit_transition(var_name: &str, transition: &str, ind: &str, level: u32, out: &mut String) {
    let parts: Vec<&str> = transition.split_whitespace().collect();
    let (prop, duration, easing) = match parts.len() {
        3 => (parts[0], parts[1].trim_end_matches('s').parse::<f32>().unwrap_or(0.3), parts[2]),
        2 => ("all", parts[0].trim_end_matches('s').parse::<f32>().unwrap_or(0.3), parts[1]),
        1 => ("all", parts[0].trim_end_matches('s').parse::<f32>().unwrap_or(0.3), "ease"),
        _ => ("all", 0.3, "ease"),
    };
    let (style, dir) = parse_easing(easing);
    let target = if prop.eq_ignore_ascii_case("opacity") || prop.eq_ignore_ascii_case("all") {
        "{ BackgroundTransparency = 0 }"
    } else if prop.eq_ignore_ascii_case("scale") {
        "{ Size = UDim2.fromScale(1, 1) }"
    } else {
        "{ BackgroundTransparency = 0 }"
    };
    let ind1 = indent(level + 1);
    out.push_str(&format!("{}do\n", ind));
    out.push_str(&format!("{}    local TweenService = game:GetService(\"TweenService\")\n", ind1));
    out.push_str(&format!("{}    local tweenInfo = TweenInfo.new({}, {}, {})\n", ind1, duration, style, dir));
    out.push_str(&format!("{}    {}.AncestryChanged:Once(function()\n", ind1, var_name));
    out.push_str(&format!("{}        if {}.Parent then\n", indent(level + 2), var_name));
    out.push_str(&format!("{}            {}.BackgroundTransparency = 1\n", indent(level + 2), var_name));
    out.push_str(&format!("{}            local tween = TweenService:Create({}, tweenInfo, {})\n", indent(level + 2), var_name, target));
    out.push_str(&format!("{}            tween:Play()\n", indent(level + 2)));
    out.push_str(&format!("{}        end\n", indent(level + 2)));
    out.push_str(&format!("{}    end)\n", ind1));
    out.push_str(&format!("{}end\n", ind));
}

fn emit_animate_on_mount(var_name: &str, animate: &str, ind: &str, level: u32, out: &mut String) {
    let parts: Vec<&str> = animate.split_whitespace().collect();
    let (name, duration, easing) = match parts.len() {
        3 => (parts[0], parts[1].trim_end_matches('s').parse::<f32>().unwrap_or(0.3), parts[2]),
        2 => ("fadeIn", parts[0].trim_end_matches('s').parse::<f32>().unwrap_or(0.3), parts[1]),
        1 => ("fadeIn", parts[0].trim_end_matches('s').parse::<f32>().unwrap_or(0.3), "ease"),
        _ => ("fadeIn", 0.3, "ease"),
    };
    let (style, dir) = parse_easing(easing);
    let target = if name.to_lowercase().contains("fade") || name.to_lowercase().contains("opacity") {
        "{ BackgroundTransparency = 0 }"
    } else if name.to_lowercase().contains("scale") {
        "{ Size = UDim2.fromScale(1, 1) }"
    } else {
        "{ BackgroundTransparency = 0 }"
    };
    let ind1 = indent(level + 1);
    let ind2 = indent(level + 2);
    out.push_str(&format!("{}do\n", ind));
    out.push_str(&format!("{}    local TweenService = game:GetService(\"TweenService\")\n", ind1));
    out.push_str(&format!("{}    local tweenInfo = TweenInfo.new({}, {}, {})\n", ind1, duration, style, dir));
    out.push_str(&format!("{}    {}.AncestryChanged:Once(function()\n", ind1, var_name));
    out.push_str(&format!("{}        if {}.Parent then\n", ind2, var_name));
    out.push_str(&format!("{}            {}.BackgroundTransparency = 1\n", ind2, var_name));
    out.push_str(&format!("{}            local tween = TweenService:Create({}, tweenInfo, {})\n", ind2, var_name, target));
    out.push_str(&format!("{}            tween:Play()\n", ind2));
    out.push_str(&format!("{}        end\n", ind2));
    out.push_str(&format!("{}    end)\n", ind1));
    out.push_str(&format!("{}end\n", ind));
}

fn collect_handlers(node: &LuauNode) -> Vec<String> {
    let mut handlers = Vec::new();
    if let Some(h) = node.properties.get("data-onclick") {
        if !handlers.contains(h) {
            handlers.push(h.clone());
        }
    }
    for child in &node.children {
        for h in collect_handlers(child) {
            if !handlers.contains(&h) {
                handlers.push(h);
            }
        }
    }
    handlers
}

pub fn generate(root: &LuauNode) -> Result<String> {
    generate_internal(root, false)
}

pub fn generate_standalone(root: &LuauNode) -> Result<String> {
    generate_internal(root, true)
}

fn generate_internal(root: &LuauNode, standalone: bool) -> Result<String> {
    let mut out = String::new();
    if standalone {
        out.push_str("local Players = game:GetService(\"Players\")\n");
        out.push_str("local player = Players.LocalPlayer\n");
        out.push_str("local playerGui = player:WaitForChild(\"PlayerGui\")\n\n");
        out.push_str("Controller = {}\n");
        for h in collect_handlers(root) {
            out.push_str(&format!("function Controller.{}()\n", h));
            out.push_str("    -- TODO: add your logic\n");
            out.push_str("end\n\n");
        }
    }
    let root_type = if root.instance_type.eq_ignore_ascii_case("ScreenGui") {
        "ScreenGui"
    } else {
        "ScreenGui"
    };
    out.push_str(&format!("local root = Instance.new(\"{}\")\n", root_type));
    out.push_str("root.Name = \"UI\"\n");
    out.push_str("root.ResetOnSpawn = false\n");
    out.push_str("root.IgnoreGuiInset = false\n\n");

    let mut counter = 0u32;
    emit_node(root, "root", &mut counter, &mut out, 0);

    if standalone {
        out.push_str("\nroot.Parent = playerGui\n");
    } else {
        out.push_str("\nreturn root\n");
    }
    Ok(out)
}
