use crate::agent::injector::inject_functions;
use crate::prelude::*;
use mlua::{Lua, TableExt, Value};
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
    lua.load(&agent_code).exec()?;

    for pair in lua.globals().pairs::<String, mlua::Table>() {
        let (_, table) = pair?;
        if table.get::<_, bool>("__is_agent").unwrap_or(false) {
            let agent_instance = table.get::<_, mlua::Table>("new")?.call::<_, mlua::Table>(())?;

            match func {
                ExecutableFunction::GetMangas => {
                    let get_mangas = agent_instance.get::<_, mlua::Function>("getMangas")?;
                    get_mangas.call::<_, Value>(())?;
                }
                ExecutableFunction::GetChapters => {
                    let get_chapters = agent_instance.get::<_, mlua::Function>("getChapters")?;
                    get_chapters.call::<_, Value>(())?;
                }
                ExecutableFunction::GetPages => {
                    let get_pages = agent_instance.get::<_, mlua::Function>("getPages")?;
                    get_pages.call::<_, Value>(())?;
                }
            }
        }
    }

    Ok(())
}