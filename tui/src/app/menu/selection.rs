use super::setting::Setting;

#[derive(Debug, Clone)]
pub struct NestedSelectionState {
    pub settings: Vec<Setting>,
    pub selected_setting: usize,
    pub selected_option: usize,
    pub focus: FocusArea,     // Track which area is focused
    pub scroll_offset: usize, // For scrolling through long lists
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusArea {
    Settings,
    Options,
}

impl NestedSelectionState {
    pub fn new(settings: Vec<Setting>) -> Self {
        Self {
            settings,
            selected_setting: 0,
            selected_option: 0,
            focus: FocusArea::Settings,
            scroll_offset: 0,
        }
    }

    pub fn next_setting(&mut self) {
        if self.settings.is_empty() {
            return;
        }
        self.selected_setting = (self.selected_setting + 1) % self.settings.len();
        self.selected_option = self.settings[self.selected_setting].selected_option;
        self.update_scroll();
    }

    pub fn previous_setting(&mut self) {
        if self.settings.is_empty() {
            return;
        }
        self.selected_setting = if self.selected_setting == 0 {
            self.settings.len() - 1
        } else {
            self.selected_setting - 1
        };
        self.selected_option = self.settings[self.selected_setting].selected_option;
        self.update_scroll();
    }

    pub fn next_option(&mut self) {
        if let Some(setting) = self.settings.get_mut(self.selected_setting) {
            setting.selected_option = (setting.selected_option + 1) % setting.options.len();
            self.selected_option = setting.selected_option;
        }
    }

    pub fn previous_option(&mut self) {
        if let Some(setting) = self.settings.get_mut(self.selected_setting) {
            setting.selected_option = if setting.selected_option == 0 {
                setting.options.len() - 1
            } else {
                setting.selected_option - 1
            };
            self.selected_option = setting.selected_option;
        }
    }

    pub fn switch_focus(&mut self) {
        self.focus = match self.focus {
            FocusArea::Settings => FocusArea::Options,
            FocusArea::Options => FocusArea::Settings,
        };
    }

    pub fn get_current_setting(&self) -> Option<&Setting> {
        self.settings.get(self.selected_setting)
    }

    pub fn _get_selected_option(&self) -> Option<String> {
        self.get_current_setting()
            .map(|setting| setting.options[setting.selected_option].clone())
    }

    fn update_scroll(&mut self) {
        // Simple scroll logic: if selected item is above/below visible area, adjust scroll
        let visible_items = 10; // Adjust based on your UI
        if self.selected_setting < self.scroll_offset {
            self.scroll_offset = self.selected_setting;
        } else if self.selected_setting >= self.scroll_offset + visible_items {
            self.scroll_offset = self.selected_setting - visible_items + 1;
        }
    }

    pub fn apply_current_setting(&mut self) {
        if let Some(setting) = self.settings.get(self.selected_setting) {
            // This is where you trigger your application logic
            let setting_name = &setting.name;
            let option_value = &setting.options[setting.selected_option];

            // Dispatch based on setting name
            match setting_name.as_str() {
                "Theme" => self.handle_theme_change(option_value),
                "Language" => self.handle_language_change(option_value),
                "Font Size" => self.handle_font_size_change(option_value),
                "Notifications" => self.handle_notifications_change(option_value),
                _ => println!("Setting '{}' changed to: {}", setting_name, option_value),
            }
        }
    }

    fn handle_theme_change(&self, theme: &str) {
        println!("Theme changed to: {}", theme);
        // Your theme application logic here
    }

    fn handle_language_change(&self, language: &str) {
        println!("Language changed to: {}", language);
        // Your language switching logic here
    }

    fn handle_font_size_change(&self, size: &str) {
        println!("Font size changed to: {}", size);
        // Your font size adjustment logic here
    }

    fn handle_notifications_change(&self, setting: &str) {
        println!("Notifications: {}", setting);
        // Your notification logic here
    }
}
