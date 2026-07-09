use anyhow::{Ok, Result};
use reqwest::blocking;
use scraper::{Html, Selector};

use crate::{client::Client, config::CONFIG, frontend::Frontend};

#[derive(Default)]
pub struct AnimeAv1SUB {
    series_links: Vec<String>,
    name: String,
}

impl Client for AnimeAv1SUB {
    fn get_animes(&mut self, query: &str) -> Result<Vec<String>> {
        let url = format!("https://animeav1.com/catalogo?search={}", query);
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
        let url = format!("https://animeav1.com{}", self.name);
        let response = blocking::get(url)?;
        let html = Html::parse_document(&response.text()?);

        let mut episodes = Vec::new();
        for article in
            html.select(&Selector::parse("article.group\\/item").expect("Invalid selector"))
        {
            let tittle = article
                .select(&Selector::parse("span").expect("Invalid selector"))
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
            episodes.push(tittle.parse()?);
        }

        Ok(episodes)
    }

    fn get_episode_link(&mut self, episode: i32) -> Result<String> {
        let link = self.default_get_episode_link(episode)?;
        if CONFIG.read().unwrap().get_frontend() != Frontend::Mpv {
            return Ok(link);
        }

        self.get_episode_link_mpv(&link)
    }
}

impl AnimeAv1SUB {
    fn default_get_episode_link(&mut self, episode: i32) -> Result<String> {
        let url = format!("https://animeav1.com{}/{}", self.name, episode);
        let response = blocking::get(url)?;
        let text = response.text()?;

        let pattern = r#"embeds:{"#;
        let start_text_idx = text.find(pattern).ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Episode link not found",
        ))? + pattern.len();

        let pattern = r#"DUB:"#;
        let start_text_idx = text[start_text_idx..]
            .find(pattern)
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Server not found. Ten en cuenta que el episodio puede no estar doblado",
            ))?
            + pattern.len()
            + start_text_idx;

        let pattern = r#"server:"MP4Upload",url:""#;
        let start_text_idx = text[start_text_idx..]
            .find(pattern)
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Server not found",
            ))?
            + pattern.len()
            + start_text_idx;

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

    fn get_episode_link_mpv(&mut self, link: &str) -> Result<String> {
        let response = blocking::get(link)?;
        let text = response.text()?;

        let pattern = r#"src: ""#;
        let start_text_idx = text.find(pattern).ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Episode link not found",
        ))? + pattern.len();

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
