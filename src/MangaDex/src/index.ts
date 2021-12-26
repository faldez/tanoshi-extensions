import * as moment from "moment";
import { Chapter, Extension, fetch, Group, Input, Manga, Select, Text, State, Checkbox, TriState } from "tanoshi-extension-lib"
import { paths, components } from './dto';
import { data as tags } from './tag.json';

type MangaListSuccess = paths["/manga"]['get']['responses'][200]['content']['application/json'];
type MangaListError = paths["/manga"]['get']['responses'][400]['content']['application/json'];
type MangaListResponse = MangaListSuccess | MangaListError;

type MangaSuccess = paths["/manga/{id}"]['get']['responses'][200]['content']['application/json'];
type MangaError = paths["/manga/{id}"]['get']['responses'][404]['content']['application/json'];
type MangaResponse = MangaSuccess | MangaError;

type MangaFeedSuccess = paths["/manga/{id}/feed"]['get']['responses'][200]['content']['application/json'];
type MangaFeedError = paths["/manga/{id}/feed"]['get']['responses'][400]['content']['application/json'];
type MangaFeedResponse = MangaFeedSuccess | MangaFeedError;

type ChapterSuccess = paths["/chapter/{id}"]['get']['responses'][200]['content']['application/json'];
type ChapterError = paths["/chapter/{id}"]['get']['responses'][404]['content']['application/json'];
type ChapterResponse = ChapterSuccess | ChapterError;

export default class MangaDex extends Extension {
    id = 2;
    name = "MangaDex";
    url = "https://api.mangadex.org";
    version = "0.1.6";
    icon = "https://mangadex.org/favicon.ico";
    languages = "all";
    nsfw = true;

    titleFilter = new Text("Title");
    authorsFilter = new Text("Author");
    artistsFilter = new Text("Artist")
    yearFilter = new Text("Year")
    tagsFilter = new Group("Tags", tags.map((tag) => new State(tag.attributes.name.en)));
    includedTagsMode = new Select("Included Tags Mode", ["AND", "OR"]);
    excludedTagsMode = new Select("Excluded Tags Mode", ["AND", "OR"]);
    statusFilter = new Group("Status", [
        new Checkbox("ongoing"),
        new Checkbox("completed"),
        new Checkbox("hiatus"),
        new Checkbox("cancelled"),
    ]);

    override getFilterList(): Input[] {
        return [
            this.titleFilter,
            this.authorsFilter,
            this.yearFilter,
            this.tagsFilter,
            this.includedTagsMode,
            this.excludedTagsMode,
            this.statusFilter
        ]
    }

    async getMangaList(page: number, query?: string): Promise<Manga[]> {
        if (page < 1) {
            page = 1;
        }
        let offset = (page - 1) * 20;
        var body: MangaListResponse = await fetch(`${this.url}/manga?limit=20&offset=${offset}&includes[]=author&includes[]=artist&includes[]=cover_art${query ? '&' + query : ''}`).then((res) => res.json());

        var manga = [];
        for (const item of (body as MangaListSuccess).data!) {
            manga.push(this.mapItemToManga(item));
        }

        return Promise.resolve(manga);
    }

    async getPopularManga(page: number): Promise<Manga[]> {
        let manga = await this.getMangaList(page, 'order[followedCount]=desc')
        return Promise.resolve(manga);
    }

    async getLatestManga(page: number): Promise<Manga[]> {
        let manga = await this.getMangaList(page);
        return Promise.resolve(manga);
    }

    parseFilter(filters: Input[]): string {
        let param = [];
        for (const input of filters) {
            switch (input.name) {
                case "Title": {
                    let s = input as Text;
                    if (s.state) {
                        param.push(`title=${s.state}`);
                    }
                    break;
                }
                case "Author": {
                    let s = input as Text;
                    if (s.state) {
                        param.push(`authors[]=${s.state}`);
                    }
                    break;
                }
                case "Artist": {
                    let s = input as Text;
                    if (s.state) {
                        param.push(`artists[]=${s.state}`);
                    }
                    break;
                }
                case "Year": {
                    let s = input as Text;
                    if (s.state) {
                        param.push(`year=${s.state}`);
                    }
                    break;
                }
                case "Tags": {
                    let s = input as Group<State>;
                    if (s.state) {
                        for (const val of s.state) {
                            let includedTags = tags.filter((tag) => tag.attributes.name.en === val.name && val.selected === TriState.Included).map((tag) => `includedTags[]=${tag.id}`);
                            param.push(...includedTags);
                            let excludedTags = tags.filter((tag) => tag.attributes.name.en === val.name && val.selected === TriState.Excluded).map((tag) => `includedTags[]=${tag.id}`);
                            param.push(...excludedTags);
                        }
                    }
                    break;
                }
                case "Included Tags Mode": {
                    let s = input as Select<string>;
                    if (s.state) {
                        param.push(`includedTagsMode=${s.state}`);
                    }
                    break;
                }
                case "Excluded Tags Mode": {
                    let s = input as Select<string>;
                    if (s.state) {
                        param.push(`excludedTagsMode=${s.state}`);
                    }
                    break;
                }
                case "Status": {
                    let s = input as Group<Checkbox>;
                    if (s.state) {
                        let status = s.state.filter((val) => val === undefined || val.state === true).map((val) => `status=${s.state}`)
                        param.push(...status);
                    }
                    break;
                }
            }
        }

        return param.join('&');
    }

    async searchManga(page: number, query?: string, filter?: Input[]): Promise<Manga[]> {
        let param = undefined;
        if (filter) {
            param = this.parseFilter(filter);
            console.error(param)
        } else if (query) {
            param = `title=${query}`;
        }
        let manga = await this.getMangaList(page, param);

        return Promise.resolve(manga);
    }

    mapItemToManga(item: any): Manga {
        let title = item.attributes?.title['en'];
        let genre = item.attributes?.tags?.map((tag: any) => {
            return tag.attributes?.name ? tag.attributes?.name['en'] : undefined;
        }).filter((tag: any) => tag != undefined);
        let coverFileName = item.relationships?.filter((x: any) => x.type === "cover_art").map((x: any) => x.attributes?.fileName)[0]
        let author = item.relationships?.filter((x: any) => x.type === "author").map((x: any) => x.attributes?.name);

        return <Manga>{
            sourceId: this.id,
            title: title ? title : '',
            author: author,
            status: item.attributes?.status,
            description: item.attributes?.description['en'],
            genre: genre,
            path: `/manga/${item.id!}`,
            coverUrl: `https://uploads.mangadex.org/covers/${item.id!}/${coverFileName}.256.jpg`,
        };
    }

    async getMangaDetail(path: string): Promise<Manga> {
        var body: MangaSuccess = await fetch(`${this.url}${path}?&includes[]=author&includes[]=artist&includes[]=cover_art`).then((res) => res.json());

        let item = body.data;
        let manga = this.mapItemToManga(item!);

        return Promise.resolve(manga);
    }

    async getChapters(path: string): Promise<Chapter[]> {
        var body: MangaFeedSuccess = await fetch(`${this.url}${path}/feed?limit=500&translatedLanguage[]=en`).then(res => res.json());
        let chapter = [];

        for (const item of body.data!) {
            let attributes = item.attributes;
            if (!attributes) {
                return Promise.reject(`emptry attributes for ${path}`);
            }

            chapter.push(<Chapter>{
                sourceId: this.id,
                title: `${attributes.volume ? `Volume ${attributes.volume} ` : ''}Chapter ${attributes.chapter ? attributes.chapter : 0}${attributes.title ? ' - ' + attributes.title : ''}`,
                path: `/chapter/${item.id}`,
                number: parseFloat(attributes.chapter ? attributes.chapter : '0.0'),
                uploaded: moment(attributes.publishAt, moment.ISO_8601).unix(),
            });
        }

        return Promise.resolve(chapter);
    }

    async getPages(path: string): Promise<string[]> {
        var body: ChapterSuccess = await fetch(`${this.url}${path}`).then(res => res.json());

        var base = await fetch(`${this.url}/at-home/server/${body.data?.id}`).then(res => res.json());

        let pages = [];
        let hash = body.data?.attributes?.hash;
        for (const item of body.data?.attributes?.data!) {
            pages.push(`${base.baseUrl}/data/${hash}/${item}`);
        }

        return Promise.resolve(pages);
    }

}