// codegen/luau.rs — Generates Luau source code from the LuauNode tree.
// Produces commented, viewport-scalable Roblox UI code.

use crate::dom::LuauNode;
use anyhow::Result;

/// Returns an indentation string for the given nesting level (4 spaces per level).
fn indent(level: u32) -> String {
    "    ".repeat(level as usize)
}

/// Escapes a Rust string into a Lua-safe double-quoted string literal.
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

/// Returns a human-readable description of a helper instance type for comments.
fn helper_comment(instance_type: &str) -> &'static str {
    match instance_type {
        "UIListLayout" => "layout controller (flex arrangement)",
        "UIPadding" => "inner padding",
        "UICorner" => "rounded corners",
        "UIGridLayout" => "grid layout controller",
        "UITableLayout" => "table layout controller",
        "UIPageLayout" => "page layout controller",
        "UIGradient" => "background gradient",
        "UIStroke" => "border stroke",
        "UIScale" => "scale modifier",
        "UIFlexItem" => "flex item sizing",
        "UIAspectRatioConstraint" => "aspect ratio constraint",
        _ => "UI modifier",
    }
}

/// Emits a single LuauNode (and its children recursively) as Luau code with comments.
fn emit_node(node: &LuauNode, parent_var: &str, var_counter: &mut u32, out: &mut String, level: u32) {
    *var_counter += 1;
    let var_name = format!("{}_{}", node.instance_type.to_lowercase(), var_counter);
    let ind = indent(level);

    // -- Comment: what HTML element this node came from
    if let Some(ref tag) = node.source_tag {
        out.push_str(&format!("{}-- {} from {}\n", ind, node.instance_type, tag));
    } else {
        out.push_str(&format!("{}-- {}: {}\n", ind, node.instance_type, node.name));
    }

    out.push_str(&format!(
        "{}local {} = Instance.new({})\n",
        ind,
        var_name,
        escape_lua_string(&node.instance_type)
    ));
    out.push_str(&format!("{}{}.Parent = {}\n", ind, var_name, parent_var));
    out.push_str(&format!("{}{}.Name = {}\n", ind, var_name, escape_lua_string(&node.name)));

    // Emit Size and key visual properties first (before helpers) so layout calculates correctly
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

    // Emit helper/constraint objects (UIListLayout, UIPadding, UICorner, etc.)
    for helper in &node.helpers {
        *var_counter += 1;
        let h_var = format!("{}_{}", helper.instance_type.to_lowercase(), var_counter);
        out.push_str(&format!(
            "{}-- {}\n",
            ind,
            helper_comment(&helper.instance_type)
        ));
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

    // Emit remaining properties (skip internal/already-handled ones)
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

    // Emit button click handler
    if node.properties.contains_key("data-onclick") {
        if let Some(handler) = node.properties.get("data-onclick") {
            out.push_str(&format!(
                "{}-- Click handler: Controller.{}()\n",
                ind, handler
            ));
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

    // Emit CSS transition (tween on mount)
    if let Some(transition) = node.properties.get("data-transition") {
        emit_transition(&var_name, transition, &ind, level, out);
    }
    if let Some(transition) = node.properties.get("Transition") {
        emit_transition(&var_name, transition, &ind, level, out);
    }

    // Emit CSS animation (animate on mount)
    if let Some(animate) = node.properties.get("data-animate") {
        emit_animate_on_mount(&var_name, animate, &ind, level, out);
    }
    if let Some(animate) = node.properties.get("Animation") {
        emit_animate_on_mount(&var_name, animate, &ind, level, out);
    }

    // Recurse into children
    for child in &node.children {
        emit_node(child, &var_name, var_counter, out, level + 1);
    }
}

/// Maps a property key+value to its Luau representation.
/// Values that are already Luau constructors (Color3, UDim2, Enum, etc.) are passed through.
/// String values are escaped into quoted literals.
fn map_property_to_luau(key: &str, value: &str) -> String {
    // Pass through values that are already valid Luau expressions
    if value.starts_with("Color3.") || value.starts_with("UDim2.") || value.starts_with("UDim.")
        || value.starts_with("Vector2.") || value.starts_with("Enum.") || value == "true" || value == "false"
    {
        return value.to_string();
    }
    // Numeric properties
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
    // String/enum properties
    match key_lower.as_str() {
        "richtext" => value.to_string(),
        "text" => escape_lua_string(value),
        "image" => escape_lua_string(value),
        "name" => escape_lua_string(value),
        "placeholdertext" => escape_lua_string(value),
        _ => escape_lua_string(value),
    }
}

/// Parses a CSS easing function name into Roblox EasingStyle + EasingDirection enums.
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

/// Emits a tween-based transition that plays when the element is added to the UI tree.
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
    out.push_str(&format!("{}-- Transition: {} {}s {}\n", ind, prop, duration, easing));
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

/// Emits a CSS animation that plays once when the element mounts.
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
    out.push_str(&format!("{}-- Animation: {} {}s {}\n", ind, name, duration, easing));
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

/// Recursively collects all data-onclick handler names from the node tree.
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

/// Emits the viewport-responsive UIScale block that makes the entire UI scale
/// proportionally based on screen resolution.
fn emit_viewport_scaling(out: &mut String) {
    out.push_str("\n-- ============================================================\n");
    out.push_str("-- Viewport Scaling\n");
    out.push_str("-- Scales the entire UI proportionally based on screen resolution.\n");
    out.push_str("-- Design base: 1920px wide. The UI will scale to fit any screen.\n");
    out.push_str("-- ============================================================\n");
    out.push_str("do\n");
    out.push_str("    local uiScale = Instance.new(\"UIScale\")\n");
    out.push_str("    uiScale.Name = \"ViewportScale\"\n");
    out.push_str("    uiScale.Parent = root\n");
    out.push_str("\n");
    out.push_str("    local camera = workspace.CurrentCamera\n");
    out.push_str("    local BASE_WIDTH = 1920 -- design resolution width in pixels\n");
    out.push_str("\n");
    out.push_str("    local function updateScale()\n");
    out.push_str("        local viewportSize = camera.ViewportSize\n");
    out.push_str("        if viewportSize.X > 0 then\n");
    out.push_str("            uiScale.Scale = viewportSize.X / BASE_WIDTH\n");
    out.push_str("        end\n");
    out.push_str("    end\n");
    out.push_str("\n");
    out.push_str("    -- Update scale on startup and whenever the viewport resizes\n");
    out.push_str("    updateScale()\n");
    out.push_str("    camera:GetPropertyChangedSignal(\"ViewportSize\"):Connect(updateScale)\n");
    out.push_str("end\n");
}

// ── Public API ──────────────────────────────────────────────

/// Generates Luau code that returns the root ScreenGui (module-style).
pub fn generate(root: &LuauNode) -> Result<String> {
    generate_internal(root, false)
}

/// Generates a standalone Luau script with service imports and Controller stubs.
pub fn generate_standalone(root: &LuauNode) -> Result<String> {
    generate_internal(root, true)
}

/// Core generation logic shared by module and standalone modes.
fn generate_internal(root: &LuauNode, standalone: bool) -> Result<String> {
    let mut out = String::new();

    // File header comment
    out.push_str("-- ==========================================================\n");
    out.push_str("-- Auto-generated Luau UI code\n");
    out.push_str("-- Generated by hluau (HTML/CSS to Luau transpiler)\n");
    out.push_str("-- Do not edit manually — regenerate from source HTML/CSS.\n");
    out.push_str("-- ==========================================================\n\n");

    if standalone {
        // Service imports
        out.push_str("-- === Services ===\n");
        out.push_str("local Players = game:GetService(\"Players\")\n");
        out.push_str("local player = Players.LocalPlayer\n");
        out.push_str("local playerGui = player:WaitForChild(\"PlayerGui\")\n\n");

        // Controller stub with handler functions
        out.push_str("-- === Controller ===\n");
        out.push_str("-- Define your button/event handlers here.\n");
        out.push_str("Controller = {}\n");
        for h in collect_handlers(root) {
            out.push_str(&format!("function Controller.{}()\n", h));
            out.push_str("    -- TODO: add your logic\n");
            out.push_str("end\n\n");
        }
    }

    // ScreenGui root
    out.push_str("-- === Root ScreenGui ===\n");
    let root_type = "ScreenGui";
    out.push_str(&format!("local root = Instance.new(\"{}\")\n", root_type));
    out.push_str("root.Name = \"UI\"\n");
    out.push_str("root.ResetOnSpawn = false\n");
    out.push_str("root.IgnoreGuiInset = false\n");

    // Viewport scaling block — makes the UI responsive
    emit_viewport_scaling(&mut out);

    out.push_str("\n-- === UI Tree ===\n");

    let mut counter = 0u32;
    emit_node(root, "root", &mut counter, &mut out, 0);

    if standalone {
        out.push_str("\n-- === Mount to PlayerGui ===\n");
        out.push_str("root.Parent = playerGui\n");
    } else {
        out.push_str("\nreturn root\n");
    }
    Ok(out)
}
