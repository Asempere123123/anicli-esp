use chrono::Local;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Read, Write};
use std::thread;

use crate::config::CONFIG;

pub fn spawn_logger<R: Read + Send + 'static>(
    name: &'static str,
    stream: R,
    stream_type: &'static str,
) {
    let log_path = CONFIG.read().unwrap().get_log_file().clone();

    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create log directory");
    }

    thread::spawn(move || {
        let mut reader = BufReader::new(stream);
        let mut line = String::new();

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .expect("Failed to open log file");

        while reader.read_line(&mut line).unwrap_or(0) > 0 {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let formatted = format!("[{}] [{}] [{}] {}", timestamp, name, stream_type, line);

            let _ = file.write_all(formatted.as_bytes());

            line.clear();
        }
    });
}
