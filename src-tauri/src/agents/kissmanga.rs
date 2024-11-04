use crate::agents::{fetch_html, Agent, AgentMeta, Element, Url};
use crate::prelude::*;
use reqwest::Client;
use std::collections::HashMap;
use async_trait::async_trait;

#[derive(Debug)]
pub struct KissManga {
    client: Client,
    url: String,
    request_options: HashMap<String, String>,
}

#[async_trait]
impl Agent for KissManga {
    fn new(client: Client) -> Self {
        let mut request_options = HashMap::new();
        request_options.insert("User-Agent".to_string(), "Mozilla/5.0".to_string());

        KissManga {
            client,
            url: "https://kissmanga.org".to_string(),
            request_options,
        }
    }

    fn representation(&self) -> AgentMeta {
        AgentMeta {
            name: "KissManga".to_string(),
            url: self.url.clone(),
            icon: None,
            tags: vec!["English".to_string(), "Webtoon".to_string(), "Experimental".to_string()],
        }
    }

    async fn get_mangas(&self) -> EResult<Vec<Element>> {
        let mut manga_list = Vec::new();
        let mut page = 1;
        let mut run = true;

        while run {
            let mangas = self.get_mangas_from_page(page).await?;
            if !mangas.is_empty() {
                manga_list.extend(mangas);
                page += 1;
            } else {
                run = false;
            }
        }

        Ok(manga_list)
    }

    async fn get_chapters(&self, manga: Element) -> EResult<Vec<Element>> {
        let uri = format!("{}/{}", self.url, manga.id);
        let data = fetch_html(
            &self.client,
            &uri,
            "div#leftside div.full div.episodeList div.full div.listing.full div div h3 a",
            Some(&self.request_options)
        ).await?;

        Ok(data.into_iter().map(|element| Element {
            id: element.clone(),
            title: element.replace(&manga.title, "").trim().to_string(),
        }).collect())
    }

    async fn get_pages(&self, chapter: Element) -> EResult<Vec<Url>> {
        let uri = format!("{}/{}", self.url, chapter.id);
        let data = fetch_html(
            &self.client,
            &uri,
            "div.barContent div.full div.full.watch_container div#centerDivVideo source",
            Some(&self.request_options)
        ).await?;

        Ok(data.into_iter().map(|element| element).collect())
    }
}

impl KissManga {
    async fn get_mangas_from_page(&self, page: usize) -> EResult<Vec<Element>> {
        let uri = format!("{}/manga_list/?page={}", self.url, page);
        let data = fetch_html(
            &self.client,
            &uri,
            "div.listing div.item_movies_in_cat div a.item_movies_link",
            Some(&self.request_options)
        ).await?;

        Ok(data.into_iter().map(|element| Element {
            id: element.clone(),
            title: element.trim().to_string(),
        }).collect())
    }
}