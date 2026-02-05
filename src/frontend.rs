use serde::{Deserialize, Serialize};

use crate::config::CONFIG;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Frontend {
    DefaultBrowser,
    Brave,
    Mpv,
}

impl Default for Frontend {
    fn default() -> Self {
        CONFIG.read().unwrap().get_frontend()
    }
}
