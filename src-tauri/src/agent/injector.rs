use crate::agent::functions::*;
use crate::prelude::*;
use mlua::{ExternalResult, Lua, Table};
use reqwest::Client;
use std::collections::HashMap;

fn convert_options(options: Option<HashMap<String, String>>) -> reqwest::RequestBuilder {
    let client = Client::new();
    let mut request = client.get(""); // Placeholder URL will be replaced later

    if let Some(options) = options {
        for (key, value) in options {
            match key.as_str() {
                "method" => {
                    request = match value.as_str() {
                        "POST" => client.post(""),
                        "PUT" => client.put(""),
                        "DELETE" => client.delete(""),
                        _ => client.get(""),
                    };
                }
                "headers" => {
                    let headers: HashMap<String, String> = serde_json::from_str(&value).unwrap();
                    for (header_key, header_value) in headers {
                        request = request.header(&header_key, &header_value);
                    }
                }
                "body" => {
                    request = request.body(value);
                }
                _ => {}
            }
        }
    }

    request
}

fn table_to_hashmap(table: Option<Table>) -> Option<HashMap<String, String>> {
    match table {
        Some(table) => {
            Some(table.pairs::<String, String>().collect::<Result<HashMap<_, _>, _>>().unwrap())
        }
        None => None
    }
}

pub fn inject_functions(lua: &Lua) -> EResult<()> {
    let globals = lua.globals();

    globals.set(
        "fetchHtml",
        lua.create_async_function(
            |_, (url, query, options): (String, String, Option<Table>)| async move {
                let options_map = table_to_hashmap(options);

                let result = fetch_html(url, query, options_map).await.into_lua_err()?;
                Ok(result)
            }
        )?,
    )?;

    globals.set(
        "fetchJson",
        lua.create_async_function(
            |_, (url, options): (String, Option<Table>)| async move {
                let options_map = table_to_hashmap(options);

                let result = fetch_json(url, options_map).await.into_lua_err()?;
                Ok(result)
            }
        )?,
    )?;

    globals.set(
        "getAbsPath",
        lua.create_function(
            |_, (src, base_url): (String, String)| {
                Ok(get_abs_path(src, base_url).into_lua_err()?)
            }
        )?,
    )?;

    globals.set(
        "getAbsLink",
        lua.create_function(
            |_, (element, base_url): (Table, String)| {
                let element_map = table_to_hashmap(Some(element));
                Ok(get_abs_link(element_map.unwrap(), base_url).into_lua_err()?)
            }
        )?,
    )?;

    Ok(())
}
