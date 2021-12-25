import { Chapter, Extension, fetch, Input, Manga, Select, Text } from "tanoshi-extension-lib"
import { Response, Result, Tag } from "./dto";


export default class NHentai extends Extension {
    id: number = 6;
    name: string = "NHentai";
    url: string = "https://nhentai.net";
    version: string = "0.1.2";
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

    tagsFilter = new Text("Tags", "");
    charactersFilter = new Text("Characters", "");

    override preferences: Input[] = [
        new Select("Language", ["Any", "English", "Japanese", "Chinese"])
    ];

    override getFilterList(): Input[] {
        return [
            this.tagsFilter,
            this.charactersFilter,
        ];
    }

    buildQuery(filters?: Input[]): string {
        let query = "";
        if (this.preferences) {
            for (const input of this.preferences) {
                switch (input.name) {
                    case 'Language': {
                        let select = input as Select<String>;
                        if (select.state) {
                            let lang = select.values[select.state];
                            if (lang !== 'Any') {
                                query += `language:${select.values[select.state]}`;
                            }
                        }
                    }
                }
            }
        }

        if (filters) {
            for (const filter of filters) {
                switch (filter.name) {
                    case 'Tags': {
                        let input = filter as Text;
                        input.state?.split(',').reduce((prev: string, current: string) => {
                            if (current.startsWith('-')) {
                                return prev += ' ' + `-tag:"${current}"`;
                            } else {
                                return prev += ' ' + `tag:"${current}"`;
                            }
                        }, query);
                        break;
                    }
                    case 'Characters': {
                        let input = filter as Text;
                        input.state?.split(',').reduce((prev: string, current: string) => {
                            if (current.startsWith('-')) {
                                return prev += ' ' + `-characters:"${current}"`;
                            } else {
                                return prev += ' ' + `characters:"${current}"`;
                            }
                        }, query);
                        break;
                    }
                }
            }
        }

        if (query === "") {
            query = '""';
        }

        return query;
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
        let pages = data.images.pages.map((p, i) => `https://i.nhentai.net/galleries/${data.media_id}/${i + 1}.${this.imageType[data.images.cover.t]}`)
        return Promise.resolve(pages);
    }

}