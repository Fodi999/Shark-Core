#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
/// Simple dialog memory storing (user, assistant) pairs.
pub struct Memory {
    dialogs: Vec<(String, String)>,
}

impl Memory {
    /// Load memory from a file (bincode). If file missing, return empty memory.
    pub fn load(path: &str) -> Self {
        match std::fs::read(path) {
            Ok(bytes) => match bincode::deserialize::<Memory>(&bytes) {
                Ok(m) => m,
                Err(_) => Memory::default(),
            },
            Err(_) => Memory::default(),
        }
    }

    /// Save memory to a file path
    pub fn save(&self, path: &str) {
        if let Ok(bytes) = bincode::serialize(self) {
            let _ = std::fs::write(path, bytes);
        }
    }

    /// Build a naive context string combining recent dialogs and the new input.
    pub fn build_context(&self, input: &str) -> String {
        // naive context: join last few dialogs + current input
        let mut parts = Vec::new();
        for (q, a) in self.dialogs.iter().rev().take(4) {
            parts.push(format!("Q:{} A:{}", q, a));
        }
        parts.push(format!("Q:{}", input));
        parts.join("\n")
    }

    /// Append a dialog pair and persist to default file.
    pub fn save_dialog(&mut self, input: &str, response: &str) {
        self.dialogs.push((input.to_string(), response.to_string()));
        // persist to default file
        self.save("memory.db");
    }
}
