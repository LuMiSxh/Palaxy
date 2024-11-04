mod mangadex;
mod kissmanga;

use crate::prelude::*;
use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Element {
    id: String,
    title: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AgentMeta {
    name: String,
    url: String,
    icon: Option<String>,
    tags: Vec<String>,
}

pub type Url = String;

pub fn initialize_agents() -> Vec<Box<dyn Agent + 'static>> {
    vec![
        Box::new(mangadex::MangaDex::new(Client::new())),
        Box::new(kissmanga::KissManga::new(Client::new())),
    ]
}

pub async fn fetch_html(client: &Client, url: &str, query: &str, options: Option<&HashMap<String, String>>) -> EResult<Vec<String>> {
    let mut request = client.get(url);

    if let Some(options) = options {
        for (key, value) in options {
            request = request.header(key, value);
        }
    }

    let response = request.send().await?.text().await?;
    let document = Html::parse_document(&response);
    let selector = Selector::parse(query).unwrap();
    let elements: Vec<String> = document.select(&selector).map(|e| e.inner_html()).collect();
    Ok(elements)
}

pub async fn fetch_json(client: &Client, url: &str, options: Option<&HashMap<String, String>>) -> EResult<HashMap<String, Value>> {
    let mut request = client.get(url);

    if let Some(options) = options {
        for (key, value) in options {
            request = request.header(key, value);
        }
    }

    let response = request.send().await?.json::<HashMap<String, Value>>().await?;
    Ok(response)
}


#[async_trait]
pub trait Agent: Send + Sync {
    fn new(client: Client) -> Self
    where
        Self: Sized
    ;

    fn representation(&self) -> AgentMeta;

    async fn get_mangas(&self) -> EResult<Vec<Element>>;

    async fn get_chapters(&self, manga: Element) -> EResult<Vec<Element>>;

    async fn get_pages(&self, chapter: Element) -> EResult<Vec<Url>>;
}
