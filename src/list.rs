use std::{
    borrow::Cow,
    time::{Duration, Instant},
};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget, Widget},
};

use crate::config::CONFIG;

const SEARCH_BUFFER_RESET_DURATION: Duration = Duration::from_millis(700);

pub struct OptionsList {
    contents: Vec<String>,
    list_state: ListState,
    focus: bool,
    search_buffer: String,
    last_time_buffer_written: Instant,
}

impl OptionsList {
    pub fn focus(&mut self) {
        self.focus = true;
    }

    pub fn defocus(&mut self) {
        self.focus = false;
    }

    pub fn set_contents(&mut self, contents: Vec<String>) {
        self.contents = contents;
        self.list_state.select_first();
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up => {
                self.clear_search_buffer();
                self.up();
            }
            KeyCode::Down => {
                self.clear_search_buffer();
                self.down();
            }
            KeyCode::Char(char) => self.search_buffer_register_char(char),
            _ => (),
        }
    }

    fn up(&mut self) {
        self.list_state.select_previous();
    }

    fn down(&mut self) {
        self.list_state.select_next();
    }

    pub fn current(&self) -> Option<usize> {
        self.list_state.selected()
    }

    pub fn current_value(&self) -> Option<&str> {
        if let Some(idx) = self.list_state.selected() {
            return Some(&self.contents[idx]);
        }
        None
    }

    fn clear_search_buffer(&mut self) {
        self.search_buffer.clear();
    }

    fn search_buffer_register_char(&mut self, char: char) {
        if self.last_time_buffer_written.elapsed() >= SEARCH_BUFFER_RESET_DURATION {
            self.clear_search_buffer();
        }

        self.search_buffer.push(char);
        self.last_time_buffer_written = Instant::now();

        // Find matches
        if let Some(found_match) = self
            .contents
            .iter()
            .position(|line| contains_ignore_ascii_case(line, &self.search_buffer))
        {
            self.list_state.select(Some(found_match));
        }
    }
}

impl Widget for &mut OptionsList {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let config = CONFIG.read().unwrap();

        let list_items = self.contents.iter().map(|line| {
            let mut line = Cow::Borrowed(line.as_str());
            if config.get_liked_animes().contains(line.as_ref()) {
                line.to_mut().push_str(" ★");
            }

            let Some(match_start) =
                find_ignore_ascii_case(line.as_ref(), self.search_buffer.as_str())
            else {
                return ListItem::new(line);
            };
            let match_end = match_start + self.search_buffer.len();

            ListItem::new(Line::from_iter([
                Span::raw(line[..match_start].to_owned()),
                Span::styled(
                    line[match_start..match_end].to_owned(),
                    Style::new()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(line[match_end..].to_owned()),
            ]))
        });

        let list = List::new(list_items)
            .highlight_symbol("> ")
            .highlight_style(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .block(
                Block::new()
                    .borders(Borders::RIGHT)
                    .border_style(Style::new().fg(match self.focus {
                        true => Color::Yellow,
                        false => Color::White,
                    })),
            );

        StatefulWidget::render(list, area, buf, &mut self.list_state);
    }
}

impl Default for OptionsList {
    fn default() -> Self {
        Self {
            contents: Default::default(),
            list_state: Default::default(),
            focus: Default::default(),
            search_buffer: Default::default(),
            last_time_buffer_written: Instant::now(),
        }
    }
}

fn contains_ignore_ascii_case(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }

    haystack
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}

fn find_ignore_ascii_case(haystack: &str, needle: &str) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }

    haystack
        .as_bytes()
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}
