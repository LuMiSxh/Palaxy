mod mangadex;

use crate::prelude::*;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Element {
    id: String,
    title: String,
}

pub type Url = String;


pub async fn fetch_html(client: &Client, url: &str, query: String, options: Option<&HashMap<String, String>>) -> EResult<Vec<String>> {
    let mut request = client.get(url);

    if let Some(options) = options {
        for (key, value) in options {
            request = request.header(key, value);
        }
    }

    let response = request.send().await?.text().await?;
    let document = Html::parse_document(&response);
    let selector = Selector::parse(&query).unwrap();
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


pub trait Agent {
    fn new(client: Client) -> Self
    where
        Self: Sized
    ;

    async fn get_mangas(&self) -> EResult<Vec<Element>>;

    async fn get_chapters(&self, manga: Element) -> EResult<Vec<Element>>;

    async fn get_pages(&self, chapter: Element) -> EResult<Vec<Url>>;
}
