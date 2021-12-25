import * as moment from "moment";
import { Chapter, Extension, fetch, Input, Manga } from "tanoshi-extension-lib";
import { Detail, Response, Series } from "./dto";

export abstract class GuyaBase extends Extension {
    async getPopularManga(page: number): Promise<Manga[]> {
        return this.searchManga(page);
    }

    async getLatestManga(page: number): Promise<Manga[]> {
        let body: Response = await fetch(`${this.url}/api/get_all_series`).then(res => res.json());
        let data = new Map(Object.keys(body).sort((a, b) => {
            if (body[a].last_updated > body[b].last_updated) {
                return 1;
            }

            if (body[a].last_updated < body[b].last_updated) {
                return -1;
            }

            return 0;
        }).map(title => [title, body[title]]));

        let manga = this.mapResToManga(page, data);

        return Promise.resolve(manga);
    }

    sortByTitle(a: string, b: string) {
        if (a > b) {
            return 1;
        }

        if (a < b) {
            return -1;
        }

        return 0;
    }

    mapResToManga(page: number, data: Map<string, Detail>, query?: string): Manga[] {
        let manga = Array.from(data).map(([title, detail]) => <Manga>{
            sourceId: this.id,
            title: title,
            author: [detail.author, detail.artist],
            description: detail.description,
            genre: [],
            path: `/api/series/${detail.slug}`,
            coverUrl: `${this.url}${detail.cover}`,
        });

        if (query) {
            manga = manga.filter(m => m.title.search(query));
        }

        if (page < 1) {
            page = 1;
        }

        let offset = (page - 1) * 20;
        manga = manga.slice(offset, offset + 20);

        return manga;
    }

    async searchManga(page: number, query?: string, filter?: Input[]): Promise<Manga[]> {
        let body: Response = await fetch(`${this.url}/api/get_all_series`).then(res => res.json());
        let data = new Map(Object.keys(body).sort(this.sortByTitle).map(title => [title, body[title]]));

        let manga = this.mapResToManga(page, data, query);

        return Promise.resolve(manga);
    }

    async getMangaDetail(path: string): Promise<Manga> {
        let body: Series = await fetch(`${this.url}${path}`).then(res => res.json());

        return Promise.resolve({
            sourceId: this.id,
            title: body.title,
            author: [body.author, body.author],
            genre: [],
            description: body.description,
            path,
            coverUrl: `${this.url}${body.cover}`
        });
    }

    async getChapters(path: string): Promise<Chapter[]> {
        let body: Series = await fetch(`${this.url}${path}`).then(res => res.json());

        let chapters = Object.keys(body.chapters).map(number => {
            let ch = body.chapters[number];
            let group = Object.keys(ch.groups)[0];
            return <Chapter>{
                sourceId: this.id,
                title: ch.title,
                path: `${path}/${number}`,
                number: parseFloat(number),
                scanlator: body.groups[group],
                uploaded: moment(ch.release_date[group], moment.ISO_8601).unix(),
            }
        })

        return Promise.resolve(chapters);

    }

    async getPages(path: string): Promise<string[]> {
        let number = path.substring(path.lastIndexOf('/') + 1, path.length);
        path = path.substring(0, path.lastIndexOf('/'));
        let body: Series = await fetch(`${this.url}${path}`).then(res => res.json());

        let ch = body.chapters[number];
        let group = Object.keys(ch.groups)[0];
        let pages = ch.groups[group].map(page => `${this.url}/media/manga/${body.slug}/chapters/${ch.folder}/${group}/${page}`)

        return Promise.resolve(pages);
    }

}