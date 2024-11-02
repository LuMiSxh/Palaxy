-- !strict
-- DO NOT MODIFY THIS FILE MANUALLY UNLESS YOU KNOW WHAT YOU ARE DOING

export type Manga = {
    id: string,
    title: string
}

export type Element = {
    [string]: string
}

export type FetchOptions = {
    method: string?,
    headers: Element?,
    body: string?,
}

export type Agent = {
    Id: string,
    Label: string,
    Url: string,
    Path: string,
    FetchOptions: FetchOptions?,
    -- Queries
    QueryMangas: string,
    QueryChapters: string,
    QueryPages: string,
    QueryMangaTitle: string,
    -- Callable Functions -> IMPORTANT: Implement them asynchronously
    getMangas: (self: Agent) -> { Manga },
    getChapters: (self: Agent, m: Manga) -> { Manga },
    getPages: (self: Agent, m: Manga) -> { string },
}

function fetchHtml(url: string, query: string?, options: FetchOptions?)
    -- This function gets replaced by Rust
    return ""
end

function fetchJson(url: string, options: FetchOptions?)
    -- This function gets replaced by Rust
    return {}
end

function getAbsPath(src: string, base_url: string)
    -- This function gets replaced by Rust
    return ""
end

function getAbsLink(element: Element, baseUrl: string)
    -- This function gets replaced by Rust
    return ""
end
