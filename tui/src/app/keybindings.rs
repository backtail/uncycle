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
    IncreaseBPM,
    DecreaseBPM,
    CycleTabs,
    RevCycleTabs,
    ToggleSequence,
    StartRecording,
    DeleteRecording,
    DoubleLoopLen,
    HalfLoopLen,
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
                key: KeyCode::Char('+'),
                description: "Increase BPM",
                action: Action::IncreaseBPM,
            },
            KeyAction {
                key: KeyCode::Char('-'),
                description: "Decrease BPM",
                action: Action::DecreaseBPM,
            },
            KeyAction {
                key: KeyCode::Tab,
                description: "Cycle Tabs",
                action: Action::CycleTabs,
            },
            KeyAction {
                key: KeyCode::BackTab,
                description: "Reverse Cycle Tabs",
                action: Action::RevCycleTabs,
            },
            KeyAction {
                key: KeyCode::Char(' '),
                description: "Start/Stop Sequence",
                action: Action::ToggleSequence,
            },
            KeyAction {
                key: KeyCode::Enter,
                description: "Start/Overdub recording",
                action: Action::StartRecording,
            },
            KeyAction {
                key: KeyCode::Backspace,
                description: "Delete recording",
                action: Action::DeleteRecording,
            },
            KeyAction {
                key: KeyCode::Char('j'),
                description: "Half Loop Length",
                action: Action::HalfLoopLen,
            },
            KeyAction {
                key: KeyCode::Char('k'),
                description: "Double Loop Length",
                action: Action::DoubleLoopLen,
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
