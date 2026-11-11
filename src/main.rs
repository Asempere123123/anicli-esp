use anyhow::Result;
use app::App;
use config::ConfigApp;

mod app;
mod frontend;
mod input;
mod list;
mod logging;
mod server;

mod animeav1;
mod animeav1dub;
mod animeflv;
mod client;

mod config;

const HELP: &str = concat!(
    "anicli-esp ",
    env!("CARGO_PKG_VERSION"),
    " - Ver anime subtitulado en Español desde el terminal

Uso: anicli-esp [opciones]

Opciones:
  -c, --config    Elegir de nuevo el reproductor de video (navegador, Brave o mpv)
  -h, --help      Mostrar esta ayuda

Teclas:
  Tab / Shift+Tab   Cambiar entre buscador, lista y servidores
  Enter             Buscar, abrir la serie o reproducir el episodio
  ↑ / ↓             Moverse por la lista
  (escribir)        En la lista, salta al primer resultado que coincida
  ← / →             En resultados o favoritos: marcar o quitar favorito ★
                    En servidores: cambiar de servidor
  Esc               Desde los episodios, volver a la lista anterior
  Ctrl+L            Ver favoritos
  Ctrl+R            Ver vistos recientemente (se posiciona en el siguiente episodio)
  Ctrl+U            Borrar el texto del buscador
  Ctrl+C            Salir
"
);

fn main() -> Result<()> {
    if std::env::args().any(|arg| arg == "-h" || arg == "--help") {
        print!("{HELP}");
        return Ok(());
    }

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
