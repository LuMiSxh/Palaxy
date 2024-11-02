use crate::agent::injector::inject_functions;
use crate::prelude::*;
use mlua::{Lua, ObjectLike, Value};
use tokio::fs::read_to_string;

pub enum ExecutableFunction {
    GetMangas,
    GetChapters,
    GetPages,
}

pub async fn run_agent(agent_path: &str, func: ExecutableFunction) -> EResult<()> {
    let lua = Lua::new();
    inject_functions(&lua)?;

    let agent_code = read_to_string(agent_path).await?;
    lua.load(&agent_code).exec().map_err(|e| Error::Lua(e.to_string()))?;

    for pair in lua.globals().pairs::<String, mlua::Table>() {
        let table = match pair {
            Ok((_, table)) => table,
            Err(e) => return Err(Error::Lua(e.to_string())),
        };

        if table.get::<bool>("__is_agent").unwrap_or(false) {
            let new_func = match table.get::<mlua::Function>("new") {
                Ok(f) => f,
                Err(e) => return Err(Error::Lua(e.to_string())),
            };

            let agent_instance = new_func
                .call::<mlua::Table>(())
                .map_err(|e| Error::Lua(e.to_string()))?;

            match func {
                ExecutableFunction::GetMangas => {
                    let get_mangas = agent_instance.get::<mlua::Function>("getMangas").map_err(|e| Error::Lua(e.to_string()))?;
                    get_mangas.call::<Value>(()).map_err(|e| Error::Lua(e.to_string()))?;
                }
                ExecutableFunction::GetChapters => {
                    let get_chapters = agent_instance.get::<mlua::Function>("getChapters").map_err(|e| Error::Lua(e.to_string()))?;
                    get_chapters.call::<Value>(()).map_err(|e| Error::Lua(e.to_string()))?;
                }
                ExecutableFunction::GetPages => {
                    let get_pages = agent_instance.get::<mlua::Function>("getPages").map_err(|e| Error::Lua(e.to_string()))?;
                    get_pages.call::<Value>(()).map_err(|e| Error::Lua(e.to_string()))?;
                }
            }
        }
    }

    Ok(())
}
