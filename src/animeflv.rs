use anyhow::{Ok, Result};
use reqwest::blocking;
use scraper::{Html, Selector};

use crate::client::Client;

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
        let url = format!(
            "https://www3.animeflv.net{}-{}",
            self.name.replace("anime", "ver"),
            episode
        );
        let response = blocking::get(url)?;
        let text = response.text()?;

        let pattern = r#""server":"sw""#;
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
}
