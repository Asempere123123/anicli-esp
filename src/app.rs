use std::process::{Command, Stdio};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::prelude::*;

use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::DefaultTerminal;

use crate::client::Client;
use crate::config::CONFIG;
use crate::input::Input;
use crate::list::OptionsList;
use crate::logging::spawn_logger;
use crate::server::Servers;

#[derive(Default)]
enum Focus {
    #[default]
    Input,
    List,
    Servers,
}

#[derive(Default)]
enum Stage {
    #[default]
    SeriesSelect,
    EpisodeSelect,
}

#[derive(Default)]
pub struct App<'a> {
    exit: bool,
    focus: Focus,
    client: Box<dyn Client>,
    stage: Stage,

    errors: Vec<String>,

    input: Input,
    list: OptionsList<'a>,
    servers: Servers,
}

impl App<'_> {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => (),
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if !self.errors.is_empty() {
            self.errors.clear();
            return;
        }

        match key_event.code {
            KeyCode::Esc | KeyCode::Char('c')
                if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.exit = true
            }
            KeyCode::BackTab => self.change_focus_backwards(),
            KeyCode::Tab => self.change_focus_forward(),
            KeyCode::Enter => self.handle_enter(),
            _other => match self.focus {
                Focus::Input => self.input.handle_key_event(key_event),
                Focus::List => self.list.handle_key_event(key_event),
                Focus::Servers => {
                    if let Some(client) = self.servers.handle_key_event(key_event) {
                        self.client = client;
                    }
                }
            },
        }
    }

    fn handle_enter(&mut self) {
        match self.focus {
            Focus::Input => self.handle_enter_input(),
            Focus::List => self.handle_enter_list(),
            _ => (),
        }
    }

    fn handle_enter_input(&mut self) {
        let animes = match self.client.get_animes(&self.input.content()) {
            Result::Ok(list) => list,
            Err(e) => {
                self.errors.push(e.to_string());
                return;
            }
        };

        self.change_focus_forward();
        self.list.set_contents(animes);
        self.input.clear();
        self.stage = Stage::SeriesSelect;
    }

    fn handle_enter_list(&mut self) {
        match self.stage {
            Stage::SeriesSelect => {
                if let Some(selected) = self.list.current() {
                    let episodes = match self.client.select_anime(selected) {
                        Result::Ok(episodes) => episodes,
                        Err(e) => {
                            self.errors.push(e.to_string());
                            return;
                        }
                    };
                    self.list
                        .set_contents(episodes.iter().map(|episode| episode.to_string()).collect());
                    self.stage = Stage::EpisodeSelect;
                }
            }
            Stage::EpisodeSelect => {
                if let Some(selected) = self.list.current_value() {
                    let episode_link = match self
                        .client
                        .get_episode_link(i32::from_str_radix(&selected, 10).unwrap())
                    {
                        Result::Ok(link) => link,
                        Err(e) => {
                            self.errors.push(e.to_string());
                            return;
                        }
                    };

                    match CONFIG.read().unwrap().get_frontend() {
                        crate::frontend::Frontend::Brave => {
                            match open::with(episode_link, "brave") {
                                Err(e) => {
                                    self.errors.push(e.to_string());
                                    return;
                                }
                                _ => (),
                            };
                        }
                        crate::frontend::Frontend::DefaultBrowser => {
                            match open::that(episode_link) {
                                Err(e) => {
                                    self.errors.push(e.to_string());
                                    return;
                                }
                                _ => (),
                            };
                        }
                        crate::frontend::Frontend::Mpv => {
                            match Command::new("mpv")
                                .args([
                                    "--ytdl-raw-options=ies=generic",
                                    "--fullscreen",
                                    &episode_link,
                                ])
                                .stdout(Stdio::piped())
                                .stderr(Stdio::piped())
                                .spawn()
                            {
                                Ok(mut cmd) => {
                                    if let Some(stdout) = cmd.stdout.take() {
                                        spawn_logger("mpv-backend", stdout, "stdout");
                                    };
                                    if let Some(stderr) = cmd.stderr.take() {
                                        spawn_logger("mpv-backend", stderr, "stderr");
                                    };
                                }
                                Err(e) => {
                                    self.errors.push(e.to_string());
                                    return;
                                }
                            };
                        }
                    }
                }
            }
        }
    }

    fn change_focus_forward(&mut self) {
        match self.focus {
            Focus::Input => {
                self.input.defocus();
                self.list.focus();
                self.focus = Focus::List;
            }
            Focus::List => {
                self.list.defocus();
                self.servers.focus();
                self.focus = Focus::Servers;
            }
            Focus::Servers => {
                self.servers.defocus();
                self.input.focus();
                self.focus = Focus::Input;
            }
        }
    }

    fn change_focus_backwards(&mut self) {
        match self.focus {
            Focus::Input => {
                self.input.defocus();
                self.servers.focus();
                self.focus = Focus::Servers;
            }
            Focus::Servers => {
                self.servers.defocus();
                self.list.focus();
                self.focus = Focus::List;
            }
            Focus::List => {
                self.list.defocus();
                self.input.focus();
                self.focus = Focus::Input;
            }
        }
    }
}

impl Widget for &mut App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Percentage(100),
            Constraint::Length(1),
            Constraint::Length(1),
        ]);
        let [search_area, options_area, _, server_area] = layout.areas(area);

        // Search bar
        self.input.render(search_area, buf);

        // List
        self.list.render(options_area, buf);

        // Server selector
        self.servers.render(server_area, buf);

        // Alerts
        if !self.errors.is_empty() {
            let popup_width = area.width / 2;
            let popup_height = area.height / 2;
            let popup_area = Rect::new(
                area.x + (area.width - popup_width) / 2,
                area.y + (area.height - popup_height) / 2,
                popup_width,
                popup_height,
            );

            let block = Block::default()
                .title("Los siguientes errores sucedieron")
                .title_bottom(Line::from("<Cualquier tecla para continuar>").centered())
                .borders(Borders::ALL)
                .bg(Color::Yellow)
                .fg(Color::Black);

            let errors_text = self
                .errors
                .iter()
                .enumerate()
                .map(|(i, s)| format!("{}. {}", i + 1, s))
                .collect::<Vec<String>>()
                .join("\n");

            Paragraph::new(errors_text)
                .alignment(Alignment::Left)
                .block(block)
                .render(popup_area, buf);
        }
    }
}
