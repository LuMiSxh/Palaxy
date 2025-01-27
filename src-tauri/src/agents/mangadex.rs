use crate::agents::{fetch_json, Agent, AgentMeta, Element, Url};
use crate::prelude::*;
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

#[derive(Debug)]
pub struct MangaDex {
    client: Client,
    api: String,
    request_options: HashMap<String, String>,
}

#[async_trait]
impl Agent for MangaDex {
    fn new(client: Client) -> Self {
        let mut request_options = HashMap::new();
        request_options.insert("x-referer".to_string(), "https://mangadex.org".to_string());
        request_options.insert(
            "x-sec-ch-ua".into(),
            "\" Not A;Brand\";v=\"99\", \"Chromium\";v=\"96\", \"Google Chrome\";v=\"96\"".into(),
        );

        MangaDex {
            client,
            api: "https://api.mangadex.org".into(),
            request_options,
        }
    }

    fn representation(&self) -> AgentMeta {
        AgentMeta {
            name: "MangaDex".into(),
            url: "https://mangadex.org".into(),
            icon: Some("https://mangadex.org/favicon.ico".into()),
            tags: vec![
                "Multilingual".into(),
                "Scanlation".into(),
                "Updates".into(),
                "Experimental".into(),
            ],
        }
    }

    async fn get_mangas(&self) -> EResult<Vec<Element>> {
        let url = "https://websites.hakuneko.download/mangadex.json";
        Ok(fetch_json(&self.client, url, Some(&self.request_options))
            .await?
            .into_iter()
            .map(|(id, title)| Element {
                id,
                title: title.to_string(),
            })
            .collect())
    }

    async fn get_chapters(&self, manga: Element) -> EResult<Vec<Element>> {
        let mut chapter_list = Vec::new();
        let mut page = 0;
        let mut run = true;

        while run {
            let chapters = self.get_chapters_from_page(&manga, page).await?;
            if !chapters.is_empty() {
                chapter_list.extend(chapters);
                page += 1;
            } else {
                run = false;
            }
        }

        chapter_list.reverse();
        Ok(chapter_list)
    }

    async fn get_pages(&self, chapter: Element) -> EResult<Vec<Url>> {
        let uri = format!("{}/at-home/server/{}", self.api, chapter.id);
        let data = fetch_json(&self.client, &uri, Some(&self.request_options)).await?;
        let base_url = data["baseUrl"].to_string();
        let hash = data["chapter"]["hash"].to_string();
        let files = data["chapter"]["data"].as_array().unwrap();

        Ok(files
            .iter()
            .map(|file| format!("{}/data/{}/{}", base_url, hash, file.to_string()))
            .collect())
    }
}

impl MangaDex {
    async fn get_chapters_from_page(&self, manga: &Element, page: usize) -> EResult<Vec<Element>> {
        let uri = format!(
            "{}/chapter?limit=100&offset={}&manga={}",
            self.api,
            100 * page,
            manga.id
        );
        let data = fetch_json(&Client::new(), &uri, Some(&self.request_options)).await?;

        let chapters = data["data"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|result| {
                let attributes = &result["attributes"];
                let mut title = String::new();

                if let Some(volume) = attributes["volume"].as_str() {
                    title.push_str(&format!("Vol.{}", self.pad_num(volume, 2)));
                }
                if let Some(chapter) = attributes["chapter"].as_str() {
                    title.push_str(&format!(" Ch.{}", self.pad_num(chapter, 4)));
                }
                if let Some(title_attr) = attributes["title"].as_str() {
                    if !title.is_empty() {
                        title.push_str(" - ");
                    }
                    title.push_str(title_attr);
                }
                if let Some(language) = attributes["translatedLanguage"].as_str() {
                    title.push_str(&format!(" ({})", language));
                }

                let groups = result["relationships"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|r| {
                        if r["type"].as_str().unwrap() == "scanlation_group" {
                            Some(r["id"].as_str().unwrap().to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                if groups.is_empty() {
                    Some(Element {
                        id: result["id"].to_string(),
                        title: title.trim().into(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(chapters)
    }

    fn pad_num(&self, number: &str, places: usize) -> String {
        let range: Vec<String> = number
            .split('-')
            .map(|chapter| {
                let chapter = chapter.trim();
                let digits = chapter.split('.').next().unwrap().len();
                format!("{}{}", "0".repeat(places.saturating_sub(digits)), chapter)
            })
            .collect();

        range.join("-")
    }
}
