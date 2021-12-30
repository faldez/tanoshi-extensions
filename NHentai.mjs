class $f963b40858b26a50$export$f5b8910cec6cf069 {
    equals(others) {
        return this.type === others.type && this.name === others.name;
    }
}
class $f963b40858b26a50$export$5f1af8db9871e1d6 extends $f963b40858b26a50$export$f5b8910cec6cf069 {
    constructor(name, state){
        super();
        this.name = name;
        this.state = state;
        this.type = 'Text';
    }
}
class $f963b40858b26a50$export$48513f6b9f8ce62d extends $f963b40858b26a50$export$f5b8910cec6cf069 {
    constructor(name1, state1){
        super();
        this.name = name1;
        this.state = state1;
        this.type = 'Checkbox';
    }
}
class $f963b40858b26a50$export$ef9b1a59e592288f extends $f963b40858b26a50$export$f5b8910cec6cf069 {
    constructor(name2, values, state2){
        super();
        this.name = name2;
        this.values = values;
        this.state = state2;
        this.type = 'Select';
    }
}
class $f963b40858b26a50$export$eb2fcfdbd7ba97d4 extends $f963b40858b26a50$export$f5b8910cec6cf069 {
    constructor(name3, state3){
        super();
        this.name = name3;
        this.state = state3;
        this.type = 'Group';
    }
}
class $f963b40858b26a50$export$d43f91ac58cde147 extends $f963b40858b26a50$export$f5b8910cec6cf069 {
    constructor(name4, values1, selection){
        super();
        this.name = name4;
        this.values = values1;
        this.selection = selection;
        this.type = 'Sort';
    }
}
var $f963b40858b26a50$export$e743037aea74f514;
(function(TriState) {
    TriState[TriState["Ignored"] = 0] = "Ignored";
    TriState[TriState["Included"] = 1] = "Included";
    TriState[TriState["Excluded"] = 2] = "Excluded";
})($f963b40858b26a50$export$e743037aea74f514 || ($f963b40858b26a50$export$e743037aea74f514 = {
}));
class $f963b40858b26a50$export$7254cc27399e90bd extends $f963b40858b26a50$export$f5b8910cec6cf069 {
    constructor(name5, selected){
        super();
        this.name = name5;
        this.selected = selected;
        this.type = 'State';
    }
}
class $f963b40858b26a50$export$eeddbf09bb970356 {
    constructor(){
        this.preferences = [];
    }
    /**
     * @returns list of input or undefined if no filters
     */ getFilterList() {
        return [];
    }
    /**
     * @returns preferences class or undefined if no preferences
     */ getPreferences() {
        return this.preferences;
    }
    /**
     * @returns 
     */ setPreferences(inputs) {
        let saved = new Map();
        for (var pref of inputs)saved.set(`${pref.type}(${pref.name})`, pref);
        this.preferences = this.preferences.map((field)=>{
            let f = saved.get(`${field.type}(${field.name})`);
            if (f) field = f;
            return field;
        });
    }
}
class $f963b40858b26a50$var$$71aeeb613c2d384f$export$9f633d56d7ec90d3 {
    constructor(headers, body){
        this.headers = headers;
        this.body = body;
    }
    json() {
        var string = this.text();
        return JSON.parse(string);
    }
    text() {
        // @ts-ignore: Unreachable code error
        var string = bytes_to_string(this.body);
        return string;
    }
}
async function $f963b40858b26a50$export$e7aa7bc5c1b3cfb3(url, options) {
    // @ts-ignore: Unreachable code error
    let res = await __native_fetch__(url, options);
    return Promise.resolve(new $f963b40858b26a50$var$$71aeeb613c2d384f$export$9f633d56d7ec90d3(res.headers, res.body));
}
function $f963b40858b26a50$export$c2d084dc44961371(msg) {
    // @ts-ignore: Unreachable code error
    __native_print__(msg);
}


class $229e2bdcbb0d391b$export$2e2bcd8739ae039 extends $f963b40858b26a50$export$eeddbf09bb970356 {
    getFilterList() {
        return this.filterLists;
    }
    mapQueryText(name, input) {
        if (input.state) return input.state.split(',').filter((current)=>current !== ''
        ).map((current)=>{
            if (current.startsWith('-')) return `-${name}:"${current}"`;
            else return `${name}:"${current}"`;
        });
        return [];
    }
    buildQuery(filters) {
        let query = [];
        let sort = '';
        if (this.preferences) {
            let lang = this.preferences[0];
            if (lang.state) {
                var state = lang.values[lang.state];
                if (state !== 'Any') query.push(`language:${state.toLowerCase()}`);
            }
        }
        if (filters) {
            for(const i in filters){
                let state = this.mapQueryText(filters[i].name.toLowerCase(), filters[i]);
                if (state.length > 0) query.push([
                    ...state
                ]);
            }
            let input = filters[4];
            sort = '&sort=' + input.values[input.selection !== undefined ? input.selection[0] : 0].toLowerCase().replace(' ', '-');
        }
        if (query.length === 0) return `""`;
        return query.join(' ') + sort;
    }
    async mapDataToManga(data) {
        let manga = data.result.map((item)=>{
            return {
                sourceId: this.id,
                title: item.title.pretty,
                author: [],
                genre: [],
                path: `/api/gallery/${item.id}`,
                coverUrl: `https://t.nhentai.net/galleries/${item.media_id}/cover.${this.imageType[item.images.cover.t]}`
            };
        });
        return Promise.resolve(manga);
    }
    async getPopularManga(page) {
        let data = await $f963b40858b26a50$export$e7aa7bc5c1b3cfb3(`${this.url}/api/galleries/search?query=${this.buildQuery()}&sort=popular&page=${page}`).then((res)=>res.json()
        );
        return this.mapDataToManga(data);
    }
    async getLatestManga(page1) {
        let data = await $f963b40858b26a50$export$e7aa7bc5c1b3cfb3(`${this.url}/api/galleries/search?query=${this.buildQuery()}&sort=date&page=${page1}`).then((res)=>res.json()
        );
        return this.mapDataToManga(data);
    }
    async searchManga(page2, query, filter) {
        console.log(`${this.url}/api/galleries/search?query=${query ? query : this.buildQuery(filter)}&page=${page2}`);
        let data = await $f963b40858b26a50$export$e7aa7bc5c1b3cfb3(`${this.url}/api/galleries/search?query=${query ? query : this.buildQuery(filter)}&page=${page2}`).then((res)=>res.json()
        );
        return this.mapDataToManga(data);
    }
    extractTags(tags) {
        let output = {
        };
        for (const tag of tags){
            if (!output[tag.type]) output[tag.type] = [];
            output[tag.type].push(tag.name);
        }
        return output;
    }
    async getMangaDetail(path) {
        let data = await $f963b40858b26a50$export$e7aa7bc5c1b3cfb3(`${this.url}${path}`).then((res)=>res.json()
        );
        let tags = this.extractTags(data.tags);
        let description = `#${data.id}\n`;
        if (tags['parody']) description = `${description}Parodies: ${tags['parody'].join(',')}\n`;
        if (tags['character']) description = `${description}Characters: ${tags['character'].join(',')}\n`;
        if (tags['language']) description = `${description}Languages: ${tags['language'].join(',')}\n`;
        if (tags['category']) description = `${description}Categories: ${tags['category'].join(',')}\n`;
        return Promise.resolve({
            sourceId: this.id,
            title: data.title.pretty,
            author: tags['artist'] ? tags['artist'] : [],
            genre: tags['tag'] ? tags['tag'] : [],
            description: description,
            path: `/api/gallery/${data.id}`,
            coverUrl: `https://t.nhentai.net/galleries/${data.media_id}/cover.${this.imageType[data.images.cover.t]}`
        });
    }
    async getChapters(path1) {
        let data = await $f963b40858b26a50$export$e7aa7bc5c1b3cfb3(`${this.url}${path1}`).then((res)=>res.json()
        );
        return Promise.resolve([
            {
                sourceId: this.id,
                title: `Chapter 1`,
                path: path1,
                number: 1,
                uploaded: data.upload_date
            }
        ]);
    }
    async getPages(path2) {
        let data = await $f963b40858b26a50$export$e7aa7bc5c1b3cfb3(`${this.url}${path2}`).then((res)=>res.json()
        );
        let pages = data.images.pages.map((p, i)=>`https://i.nhentai.net/galleries/${data.media_id}/${i + 1}.${this.imageType[p.t]}`
        );
        return Promise.resolve(pages);
    }
    constructor(...args){
        super(...args);
        this.id = 9;
        this.name = "NHentai";
        this.url = "https://nhentai.net";
        this.version = "0.1.6";
        this.icon = "https://static.nhentai.net/img/logo.090da3be7b51.svg";
        this.languages = "all";
        this.nsfw = true;
        this.imageType = {
            "j": "jpg",
            "g": "gif",
            "p": "png"
        };
        this.filterLists = [
            new $f963b40858b26a50$export$5f1af8db9871e1d6("Tag"),
            new $f963b40858b26a50$export$5f1af8db9871e1d6("Characters"),
            new $f963b40858b26a50$export$5f1af8db9871e1d6("Categories"),
            new $f963b40858b26a50$export$5f1af8db9871e1d6("Parodies"),
            new $f963b40858b26a50$export$d43f91ac58cde147("Sort", [
                "Popular",
                "Popular Week",
                "Popular Today",
                "Recent", 
            ]), 
        ];
        this.preferences = [
            new $f963b40858b26a50$export$ef9b1a59e592288f("Language", [
                "Any",
                "English",
                "Japanese",
                "Chinese"
            ])
        ];
    }
}


export {$229e2bdcbb0d391b$export$2e2bcd8739ae039 as default};
