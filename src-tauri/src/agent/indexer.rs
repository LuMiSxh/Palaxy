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
            lua.load(&content).exec().map_err(|e| Error::Lua(e.to_string()))?;

            for pair in lua.globals().pairs::<String, mlua::Table>() {
                let table = match pair { 
                    Ok((_, table)) => table,
                    Err(e) => return Err(Error::Lua(e.to_string())),
                };
                
                if table.get::<bool>("__is_agent").unwrap_or(false) {
                    let metadata = AgentMetadata {
                        id: table.get::<String>("Id").map_err(|e| Error::Lua(e.to_string()))?,
                        label: table.get::<String>("Label").map_err(|e| Error::Lua(e.to_string()))?,
                        url: table.get::<String>("Url").map_err(|e| Error::Lua(e.to_string()))?,
                        path: table.get::<String>("Path").map_err(|e| Error::Lua(e.to_string()))?,
                        query_mangas: table.get::<String>("QueryMangas").map_err(|e| Error::Lua(e.to_string()))?,
                        query_chapters: table.get::<String>("QueryChapters").map_err(|e| Error::Lua(e.to_string()))?,
                        query_pages: table.get::<String>("QueryPages").map_err(|e| Error::Lua(e.to_string()))?,
                        query_manga_title: table.get::<String>("QueryMangaTitle").map_err(|e| Error::Lua(e.to_string()))?,
                    };
                    agents.push(metadata);
                }
            }
        }
    }

    Ok(agents)
}