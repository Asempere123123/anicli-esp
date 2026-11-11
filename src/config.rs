use anyhow::{Ok, Result};
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use lazy_static::lazy_static;
use ratatui::prelude::*;
use ratatui::widgets::List;
use ratatui::widgets::ListState;
use ratatui::widgets::Paragraph;
use ratatui::DefaultTerminal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::{
    client::Client,
    frontend::Frontend,
    server::{Server, Servers},
};

lazy_static! {
    pub static ref CONFIG: RwLock<Config> = Config::empty();
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    client: Server,
    frontend: Frontend,
    log_file_path: PathBuf,
    liked_animes: BTreeSet<String>,
    // default: los config.json anteriores no tienen este campo y sin esto no cargarían
    #[serde(default)]
    history: Vec<WatchEntry>,
}

const HISTORY_LIMIT: usize = 50;

/// Último episodio abierto de una serie
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WatchEntry {
    pub name: String,
    pub server: Server,
    pub link: String,
    pub episode: i32,
}

/// Deja `entry` al principio, sin duplicar la serie y con un máximo de HISTORY_LIMIT entradas
fn push_history(history: &mut Vec<WatchEntry>, entry: WatchEntry) {
    history.retain(|e| !(e.server == entry.server && e.link == entry.link));
    history.insert(0, entry);
    history.truncate(HISTORY_LIMIT);
}

impl Config {
    fn from_file() -> Option<Self> {
        let dirs = directories::ProjectDirs::from("", "", "ani-cli-es")
            .expect("Could not get the config dir");
        let config_dir = dirs.config_dir();

        let config = std::fs::read_to_string(config_dir.join("config.json")).ok();
        if let Some(config) = config {
            return serde_json::from_str(&config).ok();
        }
        None
    }

    fn empty() -> RwLock<Self> {
        RwLock::new(Self {
            client: Server::AnimeAv1,
            frontend: Frontend::DefaultBrowser,
            log_file_path: PathBuf::new(),
            liked_animes: BTreeSet::new(),
            history: Vec::new(),
        })
    }

    pub fn get_client(&self) -> Box<dyn Client> {
        Servers::generate_current_client(&self.client)
    }

    pub fn get_server(&self) -> Server {
        self.client
    }

    pub fn set_client(&mut self, client: Server) {
        self.client = client;

        self.save();
    }

    pub fn get_log_file(&self) -> &PathBuf {
        &self.log_file_path
    }
    pub fn get_liked_animes(&self) -> &BTreeSet<String> {
        &self.liked_animes
    }

    pub fn toggle_like(&mut self, series: &str) {
        if !self.liked_animes.remove(series) {
            self.liked_animes.insert(series.to_owned());
        }

        self.save();
    }

    pub fn get_history(&self) -> &[WatchEntry] {
        &self.history
    }

    pub fn record_watch(&mut self, entry: WatchEntry) {
        push_history(&mut self.history, entry);

        self.save();
    }

    fn save(&mut self) {
        let dirs = directories::ProjectDirs::from("", "", "ani-cli-es")
            .expect("Could not get the config dir");
        let config_dir = dirs.config_dir();

        let config = serde_json::to_string_pretty(self).expect("Could not serialize config");

        std::fs::create_dir_all(config_dir).expect("Could not write config");
        std::fs::write(config_dir.join("config.json"), &config).expect("Could not write config");
    }

    pub fn get_frontend(&self) -> Frontend {
        self.frontend.clone()
    }

    fn set(&mut self, config: Self) {
        let _old_config = std::mem::replace(self, config);

        self.save();
    }
}

const FRONTENDS: [FrontendData; 3] = [FrontendData {
    frontend: Frontend::DefaultBrowser,
    name: "Navegador Predeterminado",
    description: "Utiliza el navegador predeterminado de el sistema operativo, puede mostrar anuncios"
},
FrontendData {
    frontend: Frontend::Brave,
    name: "Brave",
    description: "Utiliza Brave, que tiene un bloqueador de anuncios integrado. Necesita que brave esté instalado en el dispositivo"
},
FrontendData {
    frontend: Frontend::Mpv,
    name: "mpv (Recomendado)",
    description: "Utiliza mpv, evita todos los anuncios. Necesita que mpv esté instalado y puede tardar mas en abrir el video"
}];

struct FrontendData {
    frontend: Frontend,
    name: &'static str,
    description: &'static str,
}

#[derive(Default)]
pub struct ConfigApp {
    config: Option<Config>,
    frontend_state: ListState,
}

impl ConfigApp {
    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.config = Config::from_file();
        if self.config.is_none() || std::env::args().any(|arg| arg == "-c" || arg == "--config") {
            self.run_internal(terminal)?;
        }

        CONFIG
            .write()
            .unwrap()
            .set(self.config.ok_or(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Config should exist",
            ))?);
        Ok(())
    }

    fn run_internal(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let dirs = directories::ProjectDirs::from("", "", "ani-cli-es")
            .expect("Could not get the config dir");
        // Con --config se conservan likes e historial en vez de empezar de cero
        let (liked_animes, history) = self
            .config
            .take()
            .map(|config| (config.liked_animes, config.history))
            .unwrap_or_default();
        self.config = Some(Config {
            client: Server::AnimeAv1,
            frontend: self.run_select_frontend(terminal)?,
            log_file_path: dirs.data_dir().join("logs"),
            liked_animes,
            history,
        });

        Ok(())
    }

    fn run_select_frontend(&mut self, terminal: &mut DefaultTerminal) -> Result<Frontend> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            if let Some(frontend) = self.handle_events_frontend()? {
                return Ok(frontend);
            }
        }
    }

    fn handle_events_frontend(&mut self) -> Result<Option<Frontend>> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                return self.handle_key_event_frontend(key_event);
            }
            _ => (),
        }

        Ok(None)
    }

    fn handle_key_event_frontend(&mut self, key_event: KeyEvent) -> Result<Option<Frontend>> {
        match key_event.code {
            KeyCode::Enter => {
                if let Some(selected_idx) = self.frontend_state.selected() {
                    return Ok(Some(FRONTENDS[selected_idx].frontend.clone()));
                }
                Ok(None)
            }
            KeyCode::Up => {
                self.frontend_state.select_previous();
                Ok(None)
            }
            KeyCode::Down => {
                self.frontend_state.select_next();
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &mut ConfigApp {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([Constraint::Percentage(100), Constraint::Length(1)]);
        let [list_area, tip_area] = layout.areas(area);

        let title = Paragraph::new("Seleciona un reproductor de video");
        title.render(tip_area, buf);

        // List
        if let None = self.frontend_state.selected() {
            self.frontend_state.select_first();
        }
        let list = List::new(
            FRONTENDS
                .iter()
                .map(|frontend| frontend.name)
                .collect::<Vec<_>>(),
        )
        .highlight_symbol("> ")
        .highlight_style(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD));
        StatefulWidget::render(list, list_area, buf, &mut self.frontend_state);

        // Hints
        let hint = Paragraph::new(
            FRONTENDS[self.frontend_state.selected().unwrap_or_default()].description,
        );
        hint.render(tip_area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(link: &str, episode: i32) -> WatchEntry {
        WatchEntry {
            name: link.to_owned(),
            server: Server::AnimeAv1,
            link: link.to_owned(),
            episode,
        }
    }

    #[test]
    fn push_history_moves_series_to_front_without_duplicates() {
        let mut history = vec![entry("/a", 1), entry("/b", 3)];
        push_history(&mut history, entry("/b", 4));
        assert_eq!(history, vec![entry("/b", 4), entry("/a", 1)]);

        // Misma serie en otro servidor es otra entrada
        let mut dub = entry("/a", 2);
        dub.server = Server::AnimeAv1DUB;
        push_history(&mut history, dub.clone());
        assert_eq!(history.len(), 3);
        assert_eq!(history[0], dub);
    }

    #[test]
    fn push_history_is_capped() {
        let mut history = Vec::new();
        for i in 0..HISTORY_LIMIT + 5 {
            push_history(&mut history, entry(&format!("/{i}"), 1));
        }
        assert_eq!(history.len(), HISTORY_LIMIT);
        assert_eq!(history[0].link, format!("/{}", HISTORY_LIMIT + 4));
    }

    #[test]
    fn old_config_without_history_still_loads() {
        let json = r#"{"client":"AnimeAv1","frontend":"Mpv","log_file_path":"/tmp/x","liked_animes":["Foo"]}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.history.is_empty());
        assert!(config.liked_animes.contains("Foo"));
    }
}
