use super::setting::SettingDescription;

#[derive(Debug, Clone)]
pub struct NestedSelectionState {
    pub settings: Vec<SettingDescription>,
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
    pub fn new(settings: Vec<SettingDescription>) -> Self {
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

    pub fn get_current_setting(&self) -> Option<&SettingDescription> {
        self.settings.get(self.selected_setting)
    }

    pub fn _get_selected_option(&self) -> Option<String> {
        self.get_current_setting()
            .map(|setting| setting.options[setting.selected_option].clone())
    }

    fn update_scroll(&mut self) {
        let visible_items = 10;
        if self.selected_setting < self.scroll_offset {
            self.scroll_offset = self.selected_setting;
        } else if self.selected_setting >= self.scroll_offset + visible_items {
            self.scroll_offset = self.selected_setting - visible_items + 1;
        }
    }

    pub fn apply_current_setting(&mut self) {
        if let Some(setting) = self.settings.get(self.selected_setting) {
            (setting.apply_fn)(&setting);
        }
    }
}
