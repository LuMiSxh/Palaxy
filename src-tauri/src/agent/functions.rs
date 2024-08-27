use crate::prelude::EResult;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;

pub async fn fetch_html(url: String, query: String, options: Option<HashMap<String, String>>) -> EResult<Vec<String>> {
    let client = Client::new();
    let mut request = client.get(&url);

    if let Some(options) = options {
        for (key, value) in options {
            request = request.header(&key, &value);
        }
    }

    let response = request.send().await?.text().await?;
    let document = Html::parse_document(&response);
    let selector = Selector::parse(&query).unwrap();
    let elements: Vec<String> = document.select(&selector).map(|e| e.inner_html()).collect();
    Ok(elements)
}

pub async fn fetch_json(url: String, options: Option<HashMap<String, String>>) -> EResult<HashMap<String, String>> {
    let client = Client::new();
    let mut request = client.get(&url);

    if let Some(options) = options {
        for (key, value) in options {
            request = request.header(&key, &value);
        }
    }

    let response = request.send().await?.json::<HashMap<String, String>>().await?;
    Ok(response)
}

pub fn get_abs_path(src: String, base_url: String) -> EResult<String> {
    unimplemented!("Implement the actual logic here. src: {}, base_url: {}", src, base_url)
}

pub fn get_abs_link(element: HashMap<String, String>, base_url: String) -> EResult<String> {
    unimplemented!("Implement the actual logic here. element: {:?}, base_url: {}", element, base_url)
}
