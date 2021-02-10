-- Source details
_VERSION="0.1.2"
_NAME = "mangadex"
_BASEURL = "https://mangadex.org"
_APIBASEURL = "https://api.mangadex.org"
_LANGUAGES = {
    "sa",
    "be",
    "bg",
    "mm",
    "ct",
    "cn",
    "hk",
    "cz",
    "dk",
    "nl",
    "gb",
    "ph",
    "fi",
    "fr",
    "de",
    "gr",
    "il",
    "hu",
    "id",
    "it",
    "jp",
    "kr",
    "lt",
    "my",
    "mn",
    "no",
    "ir",
    "pl",
    "br",
    "pt",
    "ro",
    "ru",
    "rs",
    "es",
    "mx",
    "se",
    "th",
    "tr",
    "ua",
    "vn"
}

-- Mangadex specifics
_STATUS = {
    "Ongoing",
    "Completed",
    "Cancelled",
    "Hiatus"
}
_GENRES = {
	[9]="Ecchi",
	[49]="Gore",
	[50]="Sexual Violence",
	[32]="Smut",
	[1]="4-Koma",
	[42]="Adaptation",
	[43]="Anthology",
	[4]="Award Winning",
	[7]="Doujinshi",
	[48]="Fan Colored",
	[45]="Full Color",
	[36]="Long Strip",
	[47]="Official Colored",
	[21]="Oneshot",
	[46]="User Created",
	[44]="Web Comic",
	[2]="Action",
	[3]="Adventure",
	[5]="Comedy",
	[51]="Crime",
	[8]="Drama",
	[10]="Fantasy",
	[13]="Historical",
	[14]="Horror",
	[41]="Isekai",
	[52]="Magical Girls",
	[17]="Mecha",
	[18]="Medical",
	[20]="Mystery",
	[53]="Philosophical",
	[22]="Psychological",
	[23]="Romance",
	[25]="Sci-Fi",
	[28]="Shoujo Ai",
	[30]="Shounen Ai",
	[31]="Slice of Life",
	[33]="Sports",
	[54]="Superhero",
	[55]="Thriller",
	[35]="Tragedy",
	[56]="Wuxia",
	[37]="Yaoi",
	[38]="Yuri",
	[57]="Aliens",
	[58]="Animals",
	[6]="Cooking",
	[59]="Crossdressing",
	[61]="Delinquents",
	[60]="Demons",
	[62]="Genderswap",
	[63]="Ghosts",
	[11]="Gyaru",
	[12]="Harem",
	[83]="Incest",
	[65]="Loli",
	[84]="Mafia",
	[66]="Magic",
	[16]="Martial Arts",
	[67]="Military",
	[64]="Monster Girls",
	[68]="Monsters",
	[19]="Music",
	[69]="Ninja",
	[70]="Office Workers",
	[71]="Police",
	[72]="Post-Apocalyptic",
	[73]="Reincarnation",
	[74]="Reverse Harem",
	[75]="Samurai",
	[24]="School Life",
	[76]="Shota",
	[34]="Supernatural",
	[77]="Survival",
	[78]="Time Travel",
	[80]="Traditional Games",
	[79]="Vampires",
	[40]="Video Games",
	[85]="Villainess",
	[81]="Virtual Reality",
	[82]="Zombies",
}
_SORT={
    ["Title ▲"]=2,
    ["Title ▼"]=3,
    ["Last updated ▲"]=0,
    ["Last updated ▼"]=1,
    ["Comments ▲"]=4,
    ["Comments ▼"]=5,
    ["Rating ▲"]=6,
    ["Rating ▼"]=7,
    ["Views ▲"]=8,
    ["Views ▼"]=9,
    ["Follows ▲"]=10,
    ["Follows ▼"]=11
}

function name()
    return _NAME
end

function base_url()
    return _BASEURL
end

function languages()
    return _LANGUAGES
end

function version()
    return _VERSION
end

function filters()
    local f = {}
    f[1] = FilterField()
    f[1].Label = "Include tags"
    f[1].Field = "tags"
    f[1].IsMultiple = true
    f[1].Values = {}
    
    f[2] = FilterField()
    f[2].Label = "Exclude tags"
    f[2].Field = "tags"
    f[2].IsMultiple = true
    f[2].Values = {}
    
    local includeTagsValues = {}
    local excludeTagsValues = {}

    local i = 1
    for k, v in pairs(_GENRES) do
        local ti = FilterValue()
        ti.Label = "" .. v
        ti.Value = "" .. k
        includeTagsValues[i] = ti
        
        local te = FilterValue()
        te.Label = "" .. v
        te.Value = "-" .. k
        excludeTagsValues[i] = te
        
        i = i + 1
    end

    f[1].Values = includeTagsValues
    f[2].Values = excludeTagsValues

    f[3] = FilterField()
    f[3].Label = "Publication status"
    f[3].IsMultiple = true
    f[3].Field = "statuses"
    
    local publicationValues = {}

    i = 1
    for k, v in pairs(_STATUS) do
        local ti = FilterValue()
        ti.Label = "" .. v
        ti.Value = "" .. k
        publicationValues[i] = ti
        i = i + 1
    end

    f[3].Values = publicationValues
    
    f[4] = FilterField()
    f[4].Label = "Sort by"
    f[4].Field = "s"
    
    local sortByValues = {}

    i = 1
    for k, v in pairs(_SORT) do
        local ti = FilterValue()
        ti.Label = k
        ti.Value = "" .. v
        sortByValues[i] = ti
        i = i + 1
    end

    f[4].Values = sortByValues

    f[5] = FilterField()
    f[5].Label = "Manga Title"
    f[5].Field = "title"

    f[6] = FilterField()
    f[6].Label = "Author"
    f[6].Field = "author"

    f[7] = FilterField()
    f[7].Label = "Artist"
    f[7].Field = "artist"

    f[8] = FilterField()
    f[8].Label = "Tag inclusion mode"
    f[8].Field = "tag_mode_inc"

    local all = FilterValue()
    all.Label = "All"
    all.Value = "all"

    local any = FilterValue()
    any.Label = "Any"
    any.Value = "any"
    f[8].Values = {
        all,
        any
    }

    f[9] = FilterField()
    f[9].Label = "Tag exclusion mode"
    f[9].Field = "tag_mode_exc"

    f[9].Values = {
        all,
        any
    }

    return f
end

function get_latest_updates_request(page)
    return {
        method="GET",
        url=_BASEURL .. "/updates/" .. page,
        headers={
            Accept="*/*"
        }
    }
end

function get_latest_updates(body)
    local scraper = require("scraper")

    local doc, error_message = scraper.parse(body)
    local nodes, error_message = scraper.find(doc, "//a[@class='manga_title text-truncate ']")
    
    local manga = {}
    if nodes == nil then
        return manga
    end

    for i = 1, #nodes do
        local m = Manga()
        m.Title = nodes[i]:InnerText()

        local href = nodes[i]:SelectAttr("href")
        for id in string.gmatch(href, "/title/(%w+)") do
            m.Path = '/v2/manga/' .. id
            m.CoverURL = _BASEURL .. "/images/manga/" .. id .. ".thumb.jpg"
        end
        
        manga[i] = m
    end
    
    return manga
end

function get_manga_details_request(manga)
    return {
        method="GET",
        url=_APIBASEURL .. manga.Path,
        headers={
            Accept="application/json"
        }
    }
end

function get_manga_details(body)
    local json = require("json")

    local data, error_message = json.decode(body)
    
    local m = Manga()
    m.Title = data['data']['title']
    m.Description = data['data']['description']
    m.Path = "/v2/manga/" .. data['data']['id']
    m.CoverURL = data['data']['mainCover']
    m.Status = _STATUS[data['data']['publication']['status']]
    
    local authors = ""
    for k, v in pairs(data['data']['artist']) do
        authors =  authors .. "," .. v
    end
    for i=1, #data['data']['author'] do
        authors = authors .. "," .. data['data']['author'][i]
    end
    m.Authors = string.sub(authors, 2)

    local genres = ""
    for i=1, #data['data']['tags'] do
        genres = genres .. "," .. _GENRES[data['data']['tags'][i]]
    end
    m.Genres = string.sub(genres, 2)

    return m
end

function get_chapters_request(manga)
    return {
        method="GET",
        url=_APIBASEURL .. manga.Path .. "/chapters?mark_read=0",
        headers={
            Accept="application/json"
        }
    }
end

function get_chapters(body)
    local json = require("json")
    local helper = require('helper')

    local data, error_message = json.decode(body)

    local chapters = {}
    for key, value in pairs(data['data']['chapters']) do
        local c = Chapter()
        c.Number = value['chapter']
        c.Path = "/v2/chapter/" .. value['id']
        c.Title = value['title']
        c.Language = value['language']
        c.UploadedAt = helper.timestamp_to_time(value['timestamp'])
        chapters[key] = c
    end

    return chapters
end

function get_chapter_request(chapter)
    return {
        method="GET",
        url=_APIBASEURL .. chapter.Path,
        headers={
            Accept="application/json"
        }
    }
end

function get_chapter(body)
    local json = require("json")
    local helper = require('helper')

    local data, error_message = json.decode(body)

    local c = Chapter()
    c.Number = data['data']['chapter']
    c.Path = _BASEURL .. "/v2/chapter/" .. data['data']['id']
    c.Title = data['data']['title']
    c.UploadedAt = helper.timestamp_to_time(data['data']['timestamp'])
    c.Language = data['data']['language']
    
    local pages = {}
    if type(data['data']['pages']) == 'table' then
        for key, value in pairs(data['data']['pages']) do
            local p = Page()
            p.Rank = key
            p.URL = data['data']['server'] .. data['data']['hash'] .. "/" .. value
            pages[key] = p
        end
    end
    c.Pages = pages

    return c
end

function login_request(param)
    return {
        method="POST",
        url=_BASEURL .. "/ajax/actions.ajax.php?function=login",
        header={
            ["Content-Type"]="multipart/form-data",
            ["X-Requested-With"]="XMLHttpRequest"
        },
        data={
            login_username=param["username"],
            login_password=param["password"],
            two_factor=param["two_factor"],
            remember_me=param["remember_me"]
        }
    }
end

function login(header, body)
    local session = header["Set-Cookie"][2]
    local token = header["Set-Cookie"][3]

    local cookies = {}
    for w in string.gmatch(session, "([^;]+)") do
        for k, v in string.gmatch(w, "(.+)=(.+)") do
            if k == "mangadex_session" then
                cookies[1] = "mangadex_session=" .. v
                break
            end
        end
    end
    
    for w in string.gmatch(token, "([^;]+)") do
        for k, v in string.gmatch(w, "(.+)=(.+)") do
            if k == "mangadex_rememberme_token" then
                cookies[2] = "mangadex_rememberme_token=" .. v
                break
            end
        end
    end

    return {
        Cookie=cookies
    }
end

function fetch_manga_request(filters)
    local url = _BASEURL .. "/search"
    if filters ~= nil then
        url = url .. "?"
        if filters['s'] == nil or filters['s'] == "" then
            filters['s'] = _SORT["Views ▼"]
        end
        for k, v in pairs(filters) do
            if k == 'page' then
                url = url .. "p=" .. v .. "&"
            else 
                url = url .. k .. "=" .. v .. "&"
            end
        end
        url = url:gsub("&$", "")
    end

    return {
        method="GET",
        url=url
    }
end

function fetch_manga(body)
    local scraper = require("scraper")

    local doc, error_message = scraper.parse(body)
    local nodes, error_message = scraper.find(doc, '//div[@data-id]')
    
    local manga = {}
    
    if nodes == nil then
        return manga
    end
    
    for k, v in nodes() do
        local m = Manga()
        local id = v:SelectAttr("data-id")
        m.Path = '/v2/manga/' .. id
        m.CoverURL = _BASEURL .. "/images/manga/" .. id .. ".thumb.jpg"
        
        local title, error_message = scraper.find_one(v, '//a[@title]')
        m.Title = title:SelectAttr("title")
        
        manga[k] = m
    end
    
    return manga
end