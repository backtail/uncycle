use crossterm::event::KeyCode;

#[derive(Debug, Clone)]
pub struct KeyAction {
    pub key: KeyCode,
    pub description: &'static str,
    pub action: Action,
}

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    StartRecording,
    StopRecording,
    StartPlayback,
    StopPlayback,
    TogglePlayback,
    ClearLoop,
    // Add more actions as needed
}

pub struct Keybindings {
    bindings: Vec<KeyAction>,
}

impl Keybindings {
    pub fn new() -> Self {
        let bindings = vec![
            KeyAction {
                key: KeyCode::Char('q'),
                description: "Quit application",
                action: Action::Quit,
            },
            KeyAction {
                key: KeyCode::Char('Q'),
                description: "Quit application",
                action: Action::Quit,
            },
            KeyAction {
                key: KeyCode::Esc,
                description: "Quit application",
                action: Action::Quit,
            },
            KeyAction {
                key: KeyCode::Char('r'),
                description: "Start recording",
                action: Action::StartRecording,
            },
            KeyAction {
                key: KeyCode::Char('R'),
                description: "Start recording",
                action: Action::StartRecording,
            },
            KeyAction {
                key: KeyCode::Char('s'),
                description: "Stop recording and playback",
                action: Action::StopRecording,
            },
            KeyAction {
                key: KeyCode::Char('S'),
                description: "Stop recording and playback",
                action: Action::StopRecording,
            },
            KeyAction {
                key: KeyCode::Char('p'),
                description: "Start playback",
                action: Action::StartPlayback,
            },
            KeyAction {
                key: KeyCode::Char('P'),
                description: "Start playback",
                action: Action::StartPlayback,
            },
            KeyAction {
                key: KeyCode::Char(' '),
                description: "Toggle play/stop",
                action: Action::TogglePlayback,
            },
            KeyAction {
                key: KeyCode::Char('c'),
                description: "Clear loop",
                action: Action::ClearLoop,
            },
            KeyAction {
                key: KeyCode::Char('C'),
                description: "Clear loop",
                action: Action::ClearLoop,
            },
        ];

        Self { bindings }
    }

    pub fn find_action(&self, key: KeyCode) -> Option<&Action> {
        self.bindings
            .iter()
            .find(|binding| binding.key == key)
            .map(|binding| &binding.action)
    }

    pub fn get_all_bindings(&self) -> &[KeyAction] {
        &self.bindings
    }

    pub fn get_bindings_for_display(&self) -> Vec<&KeyAction> {
        // Remove duplicates for display (case-insensitive)
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();

        for binding in &self.bindings {
            let key_str = match binding.key {
                KeyCode::Char(c) => c.to_lowercase().to_string(),
                _ => format!("{:?}", binding.key),
            };

            if !seen.contains(&key_str) {
                seen.insert(key_str);
                result.push(binding);
            }
        }

        result
    }
}

// Default keybindings
impl Default for Keybindings {
    fn default() -> Self {
        Self::new()
    }
}
