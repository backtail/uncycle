#[derive(Debug, Clone)]
pub struct SettingDescription {
    pub name: String,
    pub description: String,
    pub options: Vec<String>,
    pub selected_option: usize,
    pub apply_fn: fn(&SettingDescription),
}

impl Default for SettingDescription {
    fn default() -> Self {
        Self { name: String::new(), description: String::new(), options: Vec::new(), selected_option: 0, apply_fn: nop }
    }
}

fn nop(_setting: &SettingDescription) {}