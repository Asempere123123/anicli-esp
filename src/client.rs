use anyhow::Result;
use lazy_static::lazy_static;

use crate::config::CONFIG;

lazy_static! {
    // Un solo cliente para todas las peticiones: reutiliza la conexión entre pasos
    pub static ref HTTP: reqwest::blocking::Client = reqwest::blocking::Client::new();
}

pub trait Client {
    fn get_animes(&mut self, query: &str) -> Result<Vec<String>>;
    fn select_anime(&mut self, idx: usize) -> Result<Vec<i32>>;
    /// Abre una serie por su link (el que devuelve `series_link`), sin depender de una búsqueda previa
    fn open_series(&mut self, link: &str) -> Result<Vec<i32>>;
    fn series_link(&self) -> &str;
    fn get_episode_link(&mut self, idx: i32) -> Result<String>;
}

impl Default for Box<dyn Client> {
    fn default() -> Self {
        CONFIG.read().unwrap().get_client()
    }
}
