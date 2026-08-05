#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShortcutEntry {
    pub alias: &'static str,
    pub description: &'static str,
}

impl ShortcutEntry {
    pub const fn new(alias: &'static str, description: &'static str) -> Self {
        Self { alias, description }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TriggerCatalogEntry {
    pub pattern: String,
    pub description: String,
}

impl TriggerCatalogEntry {
    pub fn new(pattern: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            description: description.into(),
        }
    }
}
