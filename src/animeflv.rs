use anyhow::{anyhow, Result};
use headless_chrome::{protocol::cdp::Network, Browser};
use reqwest::blocking;
use scraper::{Html, Selector};
use std::{
    sync::{mpsc, Arc},
    time::Duration,
};

use crate::{client::Client, config::CONFIG, frontend::Frontend};

#[derive(Default)]
pub struct AnimeFlv {
    series_links: Vec<String>,
    name: String,
}

impl Client for AnimeFlv {
    fn get_animes(&mut self, query: &str) -> Result<Vec<String>> {
        let url = format!("https://www3.animeflv.net/browse?q={query}");
        let response = blocking::get(url)?;
        let html = Html::parse_document(&response.text()?);

        self.series_links.clear();
        let mut series_names = Vec::new();

        for article in html.select(&Selector::parse("article").expect("Invalid selector")) {
            let link = article
                .select(&Selector::parse("a").expect("Invalid selector"))
                .next()
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "No link found",
                ))?
                .attr("href")
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "No link found",
                ))?;

            let tittle = article
                .select(&Selector::parse("h3").expect("Invalid selector"))
                .next()
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "No tittle found",
                ))?
                .text()
                .next()
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "No tittle found",
                ))?;

            self.series_links.push(link.to_owned());
            series_names.push(tittle.to_owned());
        }

        Ok(series_names)
    }

    fn select_anime(&mut self, idx: usize) -> Result<Vec<i32>> {
        self.name = self
            .series_links
            .get(idx)
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Invalid index",
            ))?
            .to_owned();
        let url = format!("https://www3.animeflv.net{}", self.name);
        let response = blocking::get(url)?;
        let text = response.text()?;

        let pattern = "var episodes = ";
        let start_idx = text.find(pattern).ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Episodes not found",
        ))? + pattern.len();

        let end_idx = text[start_idx..].find(";").ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "End of episodes not found",
        ))? + start_idx;

        let episodes: Vec<i32> = text[start_idx..end_idx]
            .trim_matches(&['[', ']'][..])
            .split("],[")
            .filter_map(|pair| pair.split(',').next())
            .filter_map(|num| num.parse::<i32>().ok())
            .collect();

        Ok(episodes)
    }

    fn get_episode_link(&mut self, episode: i32) -> Result<String> {
        if let Ok(link) = self.default_get_episode_link(episode) {
            return Ok(link);
        }

        self.get_episode_link_fallback(episode)
    }
}

impl AnimeFlv {
    fn default_get_episode_link(&mut self, episode: i32) -> Result<String> {
        let url = format!(
            "https://www3.animeflv.net{}-{}",
            self.name.replace("anime", "ver"),
            episode
        );
        let response = blocking::get(url)?;
        let text = response.text()?;

        let pattern;
        if CONFIG.read().unwrap().get_frontend() == Frontend::Mpv {
            pattern = r#""server":"yu""#;
        } else {
            pattern = r#""server":"sw""#;
        }
        let start_idx = text.find(pattern).ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "SW service not found",
        ))? + pattern.len();

        let pattern = r#""code":""#;
        let start_text_idx = text[start_idx..].find(pattern).ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Episode link not found",
        ))? + pattern.len()
            + start_idx;

        let pattern = "\"";
        let end_idx = text[start_text_idx..]
            .find(pattern)
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Episode link end not found",
            ))?
            + start_text_idx;

        Ok(text[start_text_idx..end_idx].to_owned().replace("\\", ""))
    }

    fn get_episode_link_fallback(&mut self, episode: i32) -> Result<String> {
        let url = format!(
            "https://www3.animeflv.net{}-{}",
            self.name.replace("anime", "ver"),
            episode
        );
        let response = blocking::get(url)?;
        let text = response.text()?;

        let pattern = r#""server":"stape""#;
        let start_idx = text.find(pattern).ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "STAPE service not found",
        ))? + pattern.len();

        let pattern = r#""code":""#;
        let start_text_idx = text[start_idx..].find(pattern).ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Episode link not found",
        ))? + pattern.len()
            + start_idx;

        let pattern = "\"";
        let end_idx = text[start_text_idx..]
            .find(pattern)
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Episode link end not found",
            ))?
            + start_text_idx;

        let initial_link = text[start_text_idx..end_idx].to_owned().replace("\\", "");

        let browser = Browser::default()?;
        let tab = browser.new_tab()?;
        let target_url = "radosgw";
        tab.call_method(Network::Enable {
            max_total_buffer_size: None,
            max_resource_buffer_size: None,
            max_post_data_size: None,
            report_direct_socket_traffic: None,
            enable_durable_messages: None,
        })?;

        let (tx, rx) = mpsc::channel();
        tab.add_event_listener(Arc::new(
            move |event: &headless_chrome::protocol::cdp::types::Event| match event {
                headless_chrome::protocol::cdp::types::Event::NetworkResponseReceived(params) => {
                    if params.params.response.url.contains(target_url) {
                        // When full url is found return it to main thread
                        let _ = tx.send(params.params.response.url.clone());
                    }
                }
                _ => {}
            },
        ))?;
        tab.navigate_to(&initial_link)?;

        rx.recv_timeout(Duration::from_secs(10))
            .map_err(|_e| anyhow!("Timed out trying to scrape the episode url from stape"))
    }
}
