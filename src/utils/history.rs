use serde::{Deserialize, Serialize};
use web_sys::window;

const HISTORY_KEY: &str = "cmd_history";
const MAX_HISTORY: usize = 50;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct CommandHistory {
    commands: Vec<String>,
}

impl CommandHistory {
    pub fn load() -> Self {
        if let Some(win) = window() {
            if let Ok(Some(storage)) = win.local_storage() {
                if let Ok(Some(json)) = storage.get_item(HISTORY_KEY) {
                    if let Ok(history) = serde_json::from_str(&json) {
                        return history;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(win) = window() {
            if let Ok(Some(storage)) = win.local_storage() {
                if let Ok(json) = serde_json::to_string(self) {
                    let _ = storage.set_item(HISTORY_KEY, &json);
                }
            }
        }
    }

    pub fn add(&mut self, cmd: String) {
        if cmd.trim().is_empty() {
            return;
        }

        // Remove strictly logic: if same as last, don't add.
        if let Some(last) = self.commands.last() {
            if last == &cmd {
                return;
            }
        }

        self.commands.push(cmd);
        if self.commands.len() > MAX_HISTORY {
            self.commands.remove(0);
        }
        self.save();
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn get_at(&self, idx: usize) -> Option<&String> {
        self.commands.get(idx)
    }
}
