use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{animeflv, client::Client, config::CONFIG};

#[derive(Default, Serialize, Deserialize, Clone, Copy)]
pub enum Server {
    #[default]
    AnimeFlv,
}

#[derive(Default)]
pub struct Servers {
    current_server: Server,

    focus: bool,
}

impl Servers {
    pub fn focus(&mut self) {
        self.focus = true;
    }

    pub fn defocus(&mut self) {
        self.focus = false;
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Option<Box<dyn Client>> {
        match key_event.code {
            KeyCode::Right => Some(self.right()),
            KeyCode::Left => Some(self.left()),
            _ => None,
        }
    }

    fn right(&mut self) -> Box<dyn Client> {
        self.current_server = match self.current_server {
            Server::AnimeFlv => Server::AnimeFlv,
        };

        CONFIG
            .write()
            .unwrap()
            .set_client(self.current_server.clone());
        Servers::generate_current_client(&self.current_server)
    }

    fn left(&mut self) -> Box<dyn Client> {
        self.current_server = match self.current_server {
            Server::AnimeFlv => Server::AnimeFlv,
        };

        CONFIG
            .write()
            .unwrap()
            .set_client(self.current_server.clone());
        Servers::generate_current_client(&self.current_server)
    }

    pub fn generate_current_client(server: &Server) -> Box<dyn Client> {
        Box::new(match server {
            Server::AnimeFlv => animeflv::AnimeFlv::default(),
        })
    }
}

impl Widget for &Servers {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.current_server {
            Server::AnimeFlv => Line::from(vec!["AnimeFlv".underlined()])
                .fg(match self.focus {
                    true => Color::Yellow,
                    false => Color::White,
                })
                .right_aligned()
                .render(area, buf),
        }
    }
}
