use crate::prelude::*;
use mlua::Lua;
use std::fs;
use tokio::fs::{read_dir, ReadDir};

#[derive(Debug)]
pub struct AgentMetadata {
    pub id: String,
    pub label: String,
    pub url: String,
    pub path: String,
    pub query_mangas: String,
    pub query_chapters: String,
    pub query_pages: String,
    pub query_manga_title: String,
}

pub async fn index_agents(directory: &str) -> EResult<Vec<AgentMetadata>> {
    let mut agents = Vec::new();
    let lua = Lua::new();
    let mut paths: ReadDir = read_dir(directory).await?;

    while let Some(entry) = paths.next_entry().await? {
        let path = entry.path();

        if path.is_file() & path.ends_with(".luau") {
            let content = fs::read_to_string(&path)?;
            lua.load(&content).exec()?;

            for pair in lua.globals().pairs::<String, mlua::Table>() {
                let (_, table) = pair?;
                if table.get::<_, bool>("__is_agent").unwrap_or(false) {
                    let metadata = AgentMetadata {
                        id: table.get::<_, String>("Id")?,
                        label: table.get::<_, String>("Label")?,
                        url: table.get::<_, String>("Url")?,
                        path: table.get::<_, String>("Path")?,
                        query_mangas: table.get::<_, String>("QueryMangas")?,
                        query_chapters: table.get::<_, String>("QueryChapters")?,
                        query_pages: table.get::<_, String>("QueryPages")?,
                        query_manga_title: table.get::<_, String>("QueryMangaTitle")?,
                    };
                    agents.push(metadata);
                }
            }
        }
    }

    Ok(agents)
}