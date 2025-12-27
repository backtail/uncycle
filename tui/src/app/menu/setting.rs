use uncycle_core::prelude::UncycleCore;

#[derive(Debug, Clone)]
pub struct Setting {
    pub name: String,
    pub description: String,
    pub options: Vec<String>,
    pub selected_option: usize,
    pub apply_fn: fn(&mut UncycleCore, &Setting),
}