use crate::prelude::*;
use mlua::{Function, Lua, Table};
use std::fs;
use std::fs::read_dir;
use std::path::PathBuf;

pub fn index_agents(directory: &PathBuf, lua: &Lua) -> EResult<Vec<AgentMetadata>> {
    let mut agents = Vec::new();

    for entry in read_dir(directory)? {
        let path = entry?.path();

        if path.is_file() && path.extension().unwrap_or(&std::ffi::OsStr::new("")) == "luau" {
            // If the filename is the same as the base ("Agent.luau"), ignore it
            if path.file_name().unwrap() == "Agent.luau" {
                continue;
            }

            let content = fs::read_to_string(&path)?;
            let agent: Table = lua.load(&content).eval()?;

            let metadata = AgentMetadata {
                id: agent.get::<String>("Id")?,
                label: agent.get::<String>("Label")?,
                url: agent.get::<String>("Url")?,
                path: agent.get::<String>("Path")?,
                query_mangas: agent.get::<String>("QueryMangas")?,
                query_chapters: agent.get::<String>("QueryChapters")?,
                query_pages: agent.get::<String>("QueryPages")?,
                query_manga_title: agent.get::<String>("QueryMangaTitle")?,
                get_mangas: agent.get::<Function>("getMangas")?,
                get_chapters: agent.get::<Function>("getChapters")?,
                get_pages: agent.get::<Function>("getPages")?,
            };
            agents.push(metadata);
        }
    }

    Ok(agents)
}