use std::collections::HashMap;

/// Represents a single Roblox UI instance node in the transpiled tree.
/// Each node maps to one `Instance.new(...)` call in the generated Luau code.
#[derive(Debug, Clone, Default)]
pub struct LuauNode {
    /// The Roblox instance class name (e.g. "Frame", "TextLabel", "TextButton")
    pub instance_type: String,
    /// The Name property assigned to this instance
    pub name: String,
    /// Key-value pairs of Roblox properties to set on this instance
    pub properties: HashMap<String, String>,
    /// Child UI nodes nested under this instance
    pub children: Vec<LuauNode>,
    /// Helper/constraint objects (UIListLayout, UIPadding, UICorner, etc.)
    pub helpers: Vec<LuauNode>,
    /// Original HTML source tag description for comment generation
    /// e.g. `<div class="panel main-panel">` or `<button class="btn">`
    pub source_tag: Option<String>,
}

impl LuauNode {
    pub fn new(instance_type: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            instance_type: instance_type.into(),
            name: name.into(),
            properties: HashMap::new(),
            children: Vec::new(),
            helpers: Vec::new(),
            source_tag: None,
        }
    }

    pub fn set_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.properties.insert(key.into(), value.into());
    }

    pub fn add_child(&mut self, child: LuauNode) {
        self.children.push(child);
    }

    pub fn add_helper(&mut self, helper: LuauNode) {
        self.helpers.push(helper);
    }
}
