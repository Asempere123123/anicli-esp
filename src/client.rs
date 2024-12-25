use anyhow::Result;

use crate::config::CONFIG;

pub trait Client {
    fn get_animes(&mut self, query: &str) -> Result<Vec<String>>;
    fn select_anime(&mut self, idx: usize) -> Result<Vec<i32>>;
    fn get_episode_link(&mut self, idx: i32) -> Result<String>;
}

impl Default for Box<dyn Client> {
    fn default() -> Self {
        CONFIG.get_client()
    }
}
