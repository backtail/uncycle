mod popup;
mod setting;
mod help;
mod selection;

pub use popup::{PopupTab, PopupMenu};
pub use selection::FocusArea;

use setting::Setting;
use selection::{NestedSelectionState};

use crate::App;

use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{List, ListItem, Block, Borders, ListState, Paragraph},
    text::{Line, Span},
    Frame,
};

pub fn render_nested_selection(f: &mut Frame, area: Rect, state: &mut NestedSelectionState) {
    // Split the area into two columns
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left panel: Settings list
    let settings_area = columns[0];
    render_settings_list(f, settings_area, state);

    // Right panel: Options for selected setting
    let options_area = columns[1];
    render_options_list(f, options_area, state);

    if let Some(setting) = state.get_current_setting() {
        let description_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(3)])
            .split(area)[1];

        render_description(f, description_area, setting);
    }
}

fn render_settings_list(f: &mut Frame, area: Rect, state: &mut NestedSelectionState) {
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

    f.render_stateful_widget(settings_list, area, &mut ListState::default());
}

fn render_options_list(f: &mut Frame, area: Rect, state: &mut NestedSelectionState) {
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

        f.render_stateful_widget(options_list, area, &mut ListState::default());
    } else {
        let placeholder = Paragraph::new("No setting selected").block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(" Options "),
        );
        f.render_widget(placeholder, area);
    }
}

fn render_description(f: &mut Frame, area: Rect, setting: &Setting) {
    let description = Paragraph::new(setting.description.clone())
        .block(
            Block::default()
                .borders(Borders::TOP)
                .title(" Description "),
        )
        .style(Style::default().fg(Color::Cyan));

    f.render_widget(description, area);
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
