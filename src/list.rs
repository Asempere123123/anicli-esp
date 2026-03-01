use std::borrow::Cow;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget, Widget},
};

use crate::config::CONFIG;

#[derive(Default)]
pub struct OptionsList {
    contents: Vec<String>,
    list_state: ListState,
    focus: bool,
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
            KeyCode::Up => self.up(),
            KeyCode::Down => self.down(),
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
}

impl Widget for &mut OptionsList {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let config = CONFIG.read().unwrap();

        let list_items = self.contents.iter().map(|line| {
            let mut line = Cow::Borrowed(line.as_str());
            if config.get_liked_animes().contains(line.as_ref()) {
                line.to_mut().push_str(" ★");
            }

            ListItem::new(line)
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
