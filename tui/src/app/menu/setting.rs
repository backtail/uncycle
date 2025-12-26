
#[derive(Debug, Clone, Default)]
pub struct Setting {
    pub name: String,
    pub description: String,
    pub options: Vec<String>,
    pub selected_option: usize,
}

