use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct LuauNode {
    pub instance_type: String,
    pub name: String,
    pub properties: HashMap<String, String>,
    pub children: Vec<LuauNode>,
    pub helpers: Vec<LuauNode>,
}

impl LuauNode {
    pub fn new(instance_type: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            instance_type: instance_type.into(),
            name: name.into(),
            properties: HashMap::new(),
            children: Vec::new(),
            helpers: Vec::new(),
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
