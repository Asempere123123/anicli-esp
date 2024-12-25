use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};
use symbols::border;

pub struct Input {
    content: Vec<char>,
    index: usize,

    focus: bool,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            content: Vec::default(),
            index: usize::default(),
            focus: true,
        }
    }
}

impl Input {
    fn handle_char(&mut self, char: char) {
        if self.index == self.content.len() {
            self.content.push(char);
        } else {
            self.content.insert(self.index, char);
        }

        self.index += 1;
    }

    fn render_content(&self) -> String {
        let mut content = String::with_capacity(self.content.len());

        for (idx, char) in self.content.iter().enumerate() {
            if idx == self.index {
                content.push_str(&format!("\x1b[4m{}\x1b[0m", char));
            } else {
                content.push(*char);
            }
        }

        if self.content.len() == self.index {
            content.push('_');
        } else {
            content.push('\u{00A0}');
        }

        content
    }

    pub fn content(&self) -> String {
        self.content.iter().collect()
    }

    fn handle_left(&mut self) {
        if self.index == 0 {
            return;
        }

        self.index -= 1;
    }

    fn handle_right(&mut self) {
        if self.index == self.content.len() {
            return;
        }

        self.index += 1;
    }

    fn handle_backspace(&mut self) {
        if self.index < 1 {
            return;
        }

        self.index -= 1;
        self.content.remove(self.index);
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(char) => self.handle_char(char),
            KeyCode::Backspace => self.handle_backspace(),
            KeyCode::Left => self.handle_left(),
            KeyCode::Right => self.handle_right(),
            _ => (),
        }
    }

    pub fn focus(&mut self) {
        self.focus = true;
    }

    pub fn defocus(&mut self) {
        self.focus = false;
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.index = 0;
    }
}

impl Widget for &Input {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(Line::from("Buscar Anime"))
            .border_set(border::PLAIN);
        let counter_text = Text::from(self.render_content());
        Paragraph::new(counter_text)
            .block(block)
            .fg(match self.focus {
                true => Color::Yellow,
                false => Color::White,
            })
            .render(area, buf);
    }
}
