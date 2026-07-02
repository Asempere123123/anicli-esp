use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{animeav1, animeflv, client::Client, config::CONFIG};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Server {
    AnimeFlv,
    AnimeAv1,
}

impl Default for Server {
    fn default() -> Self {
        CONFIG.read().unwrap().get_server()
    }
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
            Server::AnimeFlv => Server::AnimeAv1,
            Server::AnimeAv1 => Server::AnimeFlv,
        };

        CONFIG
            .write()
            .unwrap()
            .set_client(self.current_server.clone());
        Servers::generate_current_client(&self.current_server)
    }

    fn left(&mut self) -> Box<dyn Client> {
        self.current_server = match self.current_server {
            Server::AnimeFlv => Server::AnimeAv1,
            Server::AnimeAv1 => Server::AnimeFlv,
        };

        CONFIG
            .write()
            .unwrap()
            .set_client(self.current_server.clone());
        Servers::generate_current_client(&self.current_server)
    }

    pub fn generate_current_client(server: &Server) -> Box<dyn Client> {
        match server {
            Server::AnimeFlv => Box::new(animeflv::AnimeFlv::default()),
            Server::AnimeAv1 => Box::new(animeav1::AnimeAv1::default()),
        }
    }
}

impl Widget for &Servers {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let fg_color = if self.focus {
            Color::Yellow
        } else {
            Color::White
        };

        let spans = vec![
            Span::raw("AnimeFlv").fg(fg_color).add_modifier(
                if self.current_server == Server::AnimeFlv {
                    Modifier::UNDERLINED
                } else {
                    Modifier::empty()
                },
            ),
            Span::raw("  "),
            Span::raw("AnimeAv1").fg(fg_color).add_modifier(
                if self.current_server == Server::AnimeAv1 {
                    Modifier::UNDERLINED
                } else {
                    Modifier::empty()
                },
            ),
        ];

        Line::from(spans).right_aligned().render(area, buf);
    }
}
