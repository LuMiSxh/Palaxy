-- !strict

local AgentModule = require("Agent")
type Agent = AgentModule.Agent
type Manga = AgentModule.Manga
local fetchJson = AgentModule.fetchJson
local fetchHtml = AgentModule.fetchHtml

local function getMangas(self)
    local url = self.Url .. self.QueryMangas
    local response = fetchHtml(url, self.QueryMangas, self.FetchOptions)
    -- Assuming response is a table of mangas
    return response
end

local function getChapters(self, manga: Manga)
    local url = self.Url .. self.QueryChapters .. "?mangaId=" .. manga.id
    local response = fetchJson(url, self.FetchOptions)
    -- Assuming response is a table of chapters
    return response
end

local function getPages(self, manga: Manga)
    local url = self.Url .. self.QueryPages .. "?mangaId=" .. manga.id
    local response = fetchJson(url, self.FetchOptions)
    -- Assuming response is a table of pages
    return response
end

local ExampleAgent: Agent = {
    Id = "",
    Label = "",
    Url = "",
    Path = "",
    FetchOptions = nil,
    QueryMangas = "",
    QueryChapters = "",
    QueryPages = "",
    QueryMangaTitle = "",
    getMangas = getMangas,
    getChapters = getChapters,
    getPages = getPages
}

return ExampleAgent
