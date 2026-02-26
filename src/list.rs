use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::config::CONFIG;

#[derive(Default)]
pub struct OptionsList<'list_contents> {
    contents: Vec<ListItem<'list_contents>>,
    contents_texts: Vec<&'list_contents str>,
    list_state: ListState,

    focus: bool,
}

impl<'list_contents> OptionsList<'list_contents> {
    pub fn focus(&mut self) {
        self.focus = true;
    }

    pub fn defocus(&mut self) {
        self.focus = false;
    }

    pub fn set_contents(&mut self, contents: Vec<String>) {
        let contents = contents
            .into_iter()
            .map(|mut tittle| {
                if CONFIG.read().unwrap().get_liked_animes().contains(&tittle) {
                    tittle.push_str(" ★");
                }

                // Luego borro los viejos
                &*Box::leak(tittle.into_boxed_str())
            })
            .collect();

        // Borro los viejos
        for tittle in self.contents_texts.iter() {
            unsafe {
                let boxed_str: Box<str> = Box::from_raw(*tittle as *const str as *mut str);
                drop(boxed_str);
            }
        }

        self.contents_texts = contents;
        self.contents = self
            .contents_texts
            .iter()
            .map(|tittle| ListItem::new(*tittle))
            .collect();
        if !self.contents.is_empty() {
            self.list_state.select(Some(0));
        }
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

    pub fn select(&mut self, idx: Option<usize>) {
        self.list_state.select(idx);
    }

    pub fn current_value(&self) -> Option<String> {
        if let Some(idx) = self.list_state.selected() {
            let contents = self.contents_texts[idx]
                .strip_suffix(" ★")
                .unwrap_or(&self.contents_texts[idx]);
            // La referencia puede ser dropeada antes
            return Some(contents.to_owned());
        }
        None
    }

    pub fn get_contents(&self) -> Vec<String> {
        self.contents_texts
            .iter()
            .map(|entry| entry.strip_suffix(" ★").unwrap_or(entry).to_owned())
            .collect()
    }
}

impl Widget for &mut OptionsList<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let list = List::new(self.contents.clone())
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
