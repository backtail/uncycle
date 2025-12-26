use super::{NestedSelectionState, Setting, help::HelpMenu};

use ratatui::{
    prelude::*,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, Tabs},
};

#[derive(PartialEq, Clone)]
pub enum PopupTab {
    Menu,
    Help,
}

impl PopupTab {
    pub fn tab_number(&self) -> usize {
        match self {
            Self::Menu => 0,
            Self::Help => 1,
        }
    }
}

#[derive(Clone)]
pub struct PopupMenu {
    pub is_active: bool,
    pub tab: PopupTab,
    pub settings: NestedSelectionState,
    pub help: HelpMenu,
}

impl PopupMenu {
    pub fn new() -> Self {
        let settings = vec![
            Setting {
                name: "Theme".to_string(),
                description: "Choose the color theme for the application".to_string(),
                options: vec!["Dark".to_string(), "Light".to_string(), "Auto".to_string()],
                selected_option: 0,
            },
            Setting {
                name: "Language".to_string(),
                description: "Select your preferred language".to_string(),
                options: vec![
                    "English".to_string(),
                    "Spanish".to_string(),
                    "French".to_string(),
                    "German".to_string(),
                ],
                selected_option: 0,
            },
            Setting {
                name: "Font Size".to_string(),
                description: "Adjust the font size for better readability".to_string(),
                options: vec![
                    "Small".to_string(),
                    "Medium".to_string(),
                    "Large".to_string(),
                    "Extra Large".to_string(),
                ],
                selected_option: 1,
            },
            Setting {
                name: "Notifications".to_string(),
                description: "Configure notification preferences".to_string(),
                options: vec![
                    "Enabled".to_string(),
                    "Disabled".to_string(),
                    "Silent".to_string(),
                ],
                selected_option: 0,
            },
        ];

        Self {
            is_active: false,
            tab: PopupTab::Menu,
            settings: NestedSelectionState::new(settings),
            help: HelpMenu::default(),
        }
    }
}

impl Widget for PopupMenu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.is_active {
            // clear popup area
            Clear.render(area, buf);

            // popup border
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Color::Gray)
                .render(area, buf);

            Tabs::new(vec!["[m] Menu", "[?] Help"])
                .select(self.tab.tab_number())
                .padding(" ", " ")
                .style(Style::default().fg(Color::Gray))
                .highlight_style(
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
                .render(area, buf);
        }
    }
}
