import { Chapter, Extension, fetch, Input, Manga, Select, Sort, Text } from "tanoshi-extension-lib"
import { Response, Result, Tag } from "./dto";


export default class NHentai extends Extension {
    id: number = 9;
    name: string = "NHentai";
    url: string = "https://nhentai.net";
    version: string = "0.1.8";
    icon: string = "https://static.nhentai.net/img/logo.090da3be7b51.svg";
    languages: string = "all";
    nsfw: boolean = true;

    private readonly imageType: {
        [key: string]: string
    } = {
            "j": "jpg",
            "g": "gif",
            "p": "png",
        };

    public override filterList(): Input[] {
        return [
            new Text("Tag"),
            new Text("Characters"),
            new Text("Categories"),
            new Text("Parodies"),
            new Sort<String>("Sort", [
                "Popular",
                "Popular Week",
                "Popular Today",
                "Recent",
            ]),
        ]
    }

    override _preferences: Input[] = [
        new Select("Language", ["Any", "English", "Japanese", "Chinese"])
    ];

    mapQueryText(name: string, input: Text): string[] {
        if (input.state) {
            return input.state.split(',').filter((current) => current !== '').map((current: string) => {
                if (current.startsWith('-')) {
                    return `-${name}:"${current}"`;
                } else {
                    return `${name}:"${current}"`;
                }
            });
        }

        return [];
    }

    buildQuery(filters?: Input[]): string {
        let query = [];
        let sort = '';
        if (this.preferences) {
            let lang: Select<String> = this.preferences[0] as Select<String>;
            if (lang.state) {
                var state = lang.values[lang.state];
                if (state !== 'Any') {
                    query.push(`language:${state.toLowerCase()}`);
                }
            }
        }

        if (filters) {
            for (const i in filters) {
                let state = this.mapQueryText(filters[i].name.toLowerCase(), filters[i]);
                if (state.length > 0) {
                    query.push([...state]);
                }
            }

            let input = filters[4] as Sort<String>;
            sort = '&sort=' + input.values[input.selection !== undefined ? input.selection[0] : 0].toLowerCase().replace(' ', '-');
        }

        if (query.length === 0) {
            return `""`
        }

        return query.join(' ') + sort;
    }

    async mapDataToManga(data: Response): Promise<Manga[]> {
        let manga = data.result.map((item) => {
            return <Manga>{
                sourceId: this.id,
                title: item.title.pretty,
                author: [],
                genre: [],
                path: `/api/gallery/${item.id}`,
                coverUrl: `https://t.nhentai.net/galleries/${item.media_id}/cover.${this.imageType[item.images.cover.t]}`,
            }
        });
        return Promise.resolve(manga);
    }

    async getPopularManga(page: number): Promise<Manga[]> {
        let data: Response = await fetch(`${this.url}/api/galleries/search?query=${this.buildQuery()}&sort=popular&page=${page}`).then(res => res.json());
        return this.mapDataToManga(data);
    }

    async getLatestManga(page: number): Promise<Manga[]> {
        let data: Response = await fetch(`${this.url}/api/galleries/search?query=${this.buildQuery()}&sort=date&page=${page}`).then(res => res.json());
        return this.mapDataToManga(data);
    }
    async searchManga(page: number, query?: string, filter?: Input[]): Promise<Manga[]> {
        let data: Response = await fetch(`${this.url}/api/galleries/search?query=${query ? query : this.buildQuery(filter)}&page=${page}`).then(res => res.json());
        return this.mapDataToManga(data);
    }

    extractTags(tags: Tag[]): {
        [key: string]: string[]
    } {
        let output: {
            [key: string]: string[]
        } = {};
        for (const tag of tags) {
            if (!output[tag.type]) {
                output[tag.type] = []
            }
            output[tag.type].push(tag.name);
        }
        return output
    }

    async getMangaDetail(path: string): Promise<Manga> {
        let data: Result = await fetch(`${this.url}${path}`).then(res => res.json());
        let tags = this.extractTags(data.tags);

        let description = `#${data.id}\n`
        if (tags['parody']) {
            description = `${description}Parodies: ${tags['parody'].join(',')}\n`
        }
        if (tags['character']) {
            description = `${description}Characters: ${tags['character'].join(',')}\n`
        }
        if (tags['language']) {
            description = `${description}Languages: ${tags['language'].join(',')}\n`
        }
        if (tags['category']) {
            description = `${description}Categories: ${tags['category'].join(',')}\n`
        }
        return Promise.resolve({
            sourceId: this.id,
            title: data.title.pretty,
            author: tags['artist'] ? tags['artist'] : [],
            genre: tags['tag'] ? tags['tag'] : [],
            description: description,
            path: `/api/gallery/${data.id}`,
            coverUrl: `https://t.nhentai.net/galleries/${data.media_id}/cover.${this.imageType[data.images.cover.t]}`,
        })
    }
    async getChapters(path: string): Promise<Chapter[]> {
        let data: Result = await fetch(`${this.url}${path}`).then(res => res.json());
        return Promise.resolve([<Chapter>{
            sourceId: this.id,
            title: `Chapter 1`,
            path: path,
            number: 1,
            uploaded: data.upload_date,
        }])

    }
    async getPages(path: string): Promise<string[]> {
        let data: Result = await fetch(`${this.url}${path}`).then(res => res.json());
        let pages = data.images.pages.map((p, i) => `https://i.nhentai.net/galleries/${data.media_id}/${i + 1}.${this.imageType[p.t]}`)
        return Promise.resolve(pages);
    }

}