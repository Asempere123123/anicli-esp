use anyhow::Result;
use app::App;
use config::ConfigApp;

mod app;
mod frontend;
mod input;
mod list;
mod server;

mod animeflv;
mod client;

mod config;

fn main() -> Result<()> {
    color_eyre::install().expect("Could not install color eyre");

    // Config App
    let mut terminal = ratatui::init();
    let result = ConfigApp::default().run(&mut terminal);
    ratatui::restore();
    if result.is_err() {
        return result;
    }

    // App
    let mut terminal = ratatui::init();
    let result = App::default().run(&mut terminal);
    ratatui::restore();
    result
}
