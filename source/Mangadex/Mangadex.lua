-- Source details
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
        c.Uploaded = helper.timestamp_to_time(value['timestamp'])
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
    c.Uploaded = helper.timestamp_to_time(data['data']['timestamp'])
    
    local pages = {}
    for key, value in pairs(data['data']['pages']) do
        local p = Page()
        p.Rank = key
        p.URL = data['data']['server'] .. data['data']['hash'] .. "/" .. value
        pages[key] = p
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
        for k, v in pairs(filters) do
            url = url .. "&" .. k .. "=" .. v
        end
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