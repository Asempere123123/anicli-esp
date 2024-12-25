use anyhow::Result;
use app::App;

mod app;
mod input;
mod list;
mod server;

mod animeflv;
mod client;

mod config;

fn main() -> Result<()> {
    color_eyre::install().expect("Could not install color eyre");
    let mut terminal = ratatui::init();
    let result = App::default().run(&mut terminal);
    ratatui::restore();
    result
}
