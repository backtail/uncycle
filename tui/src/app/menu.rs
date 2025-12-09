use super::{keybindings::*, App};

use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    prelude::*,
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, BorderType, Borders, Clear, ListState, Paragraph, Tabs, Wrap},
};

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{List, ListItem},
    Frame,
};

#[derive(Debug, Clone)]
pub struct Setting {
    pub name: String,
    pub description: String,
    pub options: Vec<String>,
    pub selected_option: usize,
}

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

    pub fn get_selected_option(&self) -> Option<String> {
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

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up => {
                match self.focus {
                    FocusArea::Settings => self.previous_setting(),
                    FocusArea::Options => self.previous_option(),
                }
                true
            }
            KeyCode::Down => {
                match self.focus {
                    FocusArea::Settings => self.next_setting(),
                    FocusArea::Options => self.next_option(),
                }
                true
            }
            KeyCode::Left | KeyCode::Right => {
                self.switch_focus();
                true
            }
            KeyCode::Tab => {
                self.switch_focus();
                true
            }
            _ => false,
        }
    }
}

pub fn render_nested_selection(frame: &mut Frame, area: Rect, state: &mut NestedSelectionState) {
    // Split the area into two columns
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left panel: Settings list
    let settings_area = columns[0];
    render_settings_list(frame, settings_area, state);

    // Right panel: Options for selected setting
    let options_area = columns[1];
    render_options_list(frame, options_area, state);

    // Optional: Add description area at the bottom
    if let Some(setting) = state.get_current_setting() {
        let description_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(3)])
            .split(area)[1];

        render_description(frame, description_area, setting);
    }
}

fn render_settings_list(frame: &mut Frame, area: Rect, state: &mut NestedSelectionState) {
    let focus_style = if state.focus == FocusArea::Settings {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let border_style = if state.focus == FocusArea::Settings {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let settings_items: Vec<ListItem> = state
        .settings
        .iter()
        .enumerate()
        .map(|(i, setting)| {
            let is_selected = i == state.selected_setting;
            let indicator = if is_selected { "▶ " } else { "  " };

            let mut content = vec![
                Span::styled(indicator, focus_style),
                Span::styled(&setting.name, focus_style),
            ];

            // Show currently selected option next to setting name
            content.push(Span::styled(
                format!(": {}", setting.options[setting.selected_option]),
                Style::default().fg(Color::Green),
            ));

            let item_style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(content)).style(item_style)
        })
        .collect();

    let settings_list = List::new(settings_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" Settings (←/→ to focus) ")
            .title_style(focus_style),
    );

    frame.render_stateful_widget(settings_list, area, &mut ListState::default());
}

fn render_options_list(frame: &mut Frame, area: Rect, state: &mut NestedSelectionState) {
    let focus_style = if state.focus == FocusArea::Options {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let border_style = if state.focus == FocusArea::Options {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    if let Some(setting) = state.get_current_setting() {
        let options_items: Vec<ListItem> = setting
            .options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let is_selected = i == setting.selected_option;
                let indicator = if is_selected {
                    if state.focus == FocusArea::Options {
                        "● "
                    } else {
                        "○ "
                    }
                } else {
                    "  "
                };

                let content = vec![
                    Span::styled(indicator, focus_style),
                    Span::styled(option, focus_style),
                ];

                let item_style = if is_selected {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(content)).style(item_style)
            })
            .collect();

        let options_list = List::new(options_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(format!(" Options for: {} ", setting.name))
                .title_style(focus_style),
        );

        frame.render_stateful_widget(options_list, area, &mut ListState::default());
    } else {
        let placeholder = Paragraph::new("No setting selected").block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(" Options "),
        );
        frame.render_widget(placeholder, area);
    }
}

fn render_description(frame: &mut Frame, area: Rect, setting: &Setting) {
    let description = Paragraph::new(setting.description.clone())
        .block(
            Block::default()
                .borders(Borders::TOP)
                .title(" Description "),
        )
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(description, area);
}

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

#[derive(Default, Clone)]
pub struct HelpMenu {
    keybindings: Keybindings,
}

impl Widget for HelpMenu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let controls_text: Vec<Line> = self
            .keybindings
            .get_bindings_for_display()
            .iter()
            .map(|binding| {
                let key_str = match binding.key {
                    KeyCode::Char(' ') => "[Space]".to_string(),
                    KeyCode::Char(c) => format!("[{}]", c.to_uppercase()),
                    _ => format!("[{:?}]", binding.key),
                };
                Line::from(format!("{} - {}", key_str, binding.description))
            })
            .collect();

        let controls_block = Block::default();

        Paragraph::new(controls_text)
            .block(controls_block)
            .wrap(Wrap { trim: true })
            .render(area, buf);
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

pub fn render_popup_menu(f: &mut Frame, app: &mut App, area: Rect) {
    if app.menu.is_active {
        // gray out background
        f.render_widget(Block::new().fg(Color::DarkGray), area);

        let popup_area = area.inner(Margin {
            horizontal: area.width / 6,
            vertical: area.height / 7,
        });

        f.render_widget(app.menu.clone(), popup_area);

        let tab_area = popup_area.inner(Margin {
            horizontal: 2,
            vertical: 2,
        });

        match app.menu.tab {
            PopupTab::Menu => render_nested_selection(f, tab_area, &mut app.menu.settings),
            PopupTab::Help => f.render_widget(app.menu.help.clone(), tab_area),
        }
    }
}
