import * as moment from "moment";
import { Chapter, Extension, fetch, Group, Input, Manga, Select, Text } from "tanoshi-extension-lib"
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
    version = "0.1.0";
    icon = "https://mangadex.org/favicon.ico";
    languages = "all";
    nsfw = true;

    titleFilter = new Text("title", "");
    authorsFilter = new Text("author", "comma seperataed string");
    artistsFilter = new Text("artist", "comma seperataed string")
    yearFilter = new Text("year", "year of release")
    includeTagsFilter = new Group("included tags", tags.map((tag) => tag.attributes.name.en));
    includedTagsMode = new Select("included tags mode", ["AND", "OR"]);
    excludeTagsFilter = new Group("excluded tags", tags.map((tag) => tag.attributes.name.en));
    excludedTagsMode = new Select("excluded tags mode", ["AND", "OR"]);
    statusFilter = new Group("status", ["ongoing", "completed", "hiatus", "cancelled"]);

    getFilterList(): Input[] {
        return [
            this.titleFilter,
            this.authorsFilter,
            this.yearFilter,
            this.includeTagsFilter,
            this.includedTagsMode,
            this.excludeTagsFilter,
            this.excludedTagsMode,
            this.statusFilter
        ]
    }
    getPreferences(): Input[] {
        throw new Error("Method not implemented.");
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

    parseFilter(filter: Input[]): string {
        let param = [];
        for (const input of filter) {
            switch (input.name) {
                case "title": {
                    let s = input as Text;
                    param.push(`${s.name}=${s.state!}`);
                    break;
                }
                case "author": {
                    let s = input as Text;
                    param.push(`${s.name}=${s.state!}`);
                    break;
                }
                case "artist": {
                    let s = input as Text;
                    param.push(`${s.name}=${s.state!}`);
                    break;
                }
                case "year": {
                    let s = input as Text;
                    param.push(`${s.name}=${s.state!}`);
                    break;
                }
                case "included tags": {
                    let s = input as Group<string>;
                    for (const val of s.state!) {
                        let uuid = tags.filter((tag) => tag.attributes.name.en === val).map((tag) => tag.id)[0];
                        param.push(`includedTags[]=${uuid}`);
                    }
                    break;
                }
                case "included tags mode": {
                    let s = input as Select<string>;
                    param.push(`includedTagsMode=${s.state!}`);
                    break;
                }
                case "excluded tags": {
                    let s = input as Group<string>;
                    for (const val of s.state!) {
                        let uuid = tags.filter((tag) => tag.attributes.name.en === val).map((tag) => tag.id)[0];
                        param.push(`excludedTags[]=${uuid}`);
                    }
                    break;
                }
                case "excluded tags mode": {
                    let s = input as Select<string>;
                    param.push(`excludedTagsMode=${s.state!}`);
                    break;
                }
                case "status": {
                    let s = input as Group<string>;
                    for (const val of s.state!) {
                        let uuid = tags.filter((tag) => tag.attributes.name.en === val).map((tag) => tag.id)[0];
                        param.push(`${s.name}[]=${uuid}`);
                    }
                    break;
                }
            }
        }

        return param.join('&');
    }

    async searchManga(page: number, query?: string, filter?: Input[]): Promise<Manga[]> {
        let param;
        if (filter) {
            param = this.parseFilter(filter);
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
            chapter.push(<Chapter>{
                sourceId: this.id,
                title: item.attributes?.title,
                path: `/chapter/${item.id}`,
                number: parseFloat(item.attributes?.chapter!),
                uploaded: moment(item.attributes?.publishAt, moment.ISO_8601).unix(),
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