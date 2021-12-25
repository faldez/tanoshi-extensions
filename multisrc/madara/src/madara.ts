import { Chapter, Extension, fetch, Input, Manga } from 'tanoshi-extension-lib';
import * as cheerio from 'cheerio';
import * as moment from 'moment';

export abstract class Madara extends Extension {
    parseMangaList(body: string): Manga[] {
        const $ = cheerio.load(body);

        let elements = $('.manga-item a[href^="/webtoon"] img').toArray();

        return elements.map((el) => <Manga>{
            sourceId: this.id,
            title: $(el).parent().attr('title'),
            author: [],
            genre: [],
            path: $(el).parent().attr('href'),
            coverUrl: $(el).attr('data-src')
        });
    }

    async getPopularManga(page: number): Promise<Manga[]> {
        if (page < 1) {
            page = 1;
        }

        let body = await fetch(`${this.url}/webtoons/${page}?orderby=trending`).then(res => res.text());

        let manga = this.parseMangaList(body);
        return Promise.resolve(manga);
    }
    async getLatestManga(page: number): Promise<Manga[]> {
        if (page < 1) {
            page = 1;
        }

        let body = await fetch(`${this.url}/webtoons/${page}?orderby=latest`).then(res => res.text());

        let manga = this.parseMangaList(body);
        return Promise.resolve(manga);
    }
    async searchManga(page: number, query?: string, filter?: Input[]): Promise<Manga[]> {
        if (page < 1) {
            page = 1;
        }

        let body = await fetch(`${this.url}/search?q=${query}&page=${page}`).then(res => res.text());
        let manga = this.parseMangaList(body);
        return Promise.resolve(manga);
    }
    async getMangaDetail(path: string): Promise<Manga> {
        let body = await fetch(`${this.url}${path}`).then(res => res.text());
        const $ = cheerio.load(body);

        let el = $('a[href^="/webtoon"] img');
        return Promise.resolve(<Manga>{
            sourceId: this.id,
            title: el.attr('title'),
            author: $('.artist-content a').toArray().map((el) => $(el).text()!),
            genre: $('.genres-content a').toArray().map((el) => $(el).text()!),
            description: $('.dsct > p').text(),
            path: path,
            coverUrl: el.attr('data-src')
        })
    }
    async getChapters(path: string): Promise<Chapter[]> {
        let body = await fetch(`${this.url}${path}`).then(res => res.text());
        const $ = cheerio.load(body);
        let chapters = $('#chapterlist .a-h.wleft').toArray().map((el) => {
            let chapterName = $(el).children('a');

            let chapterTime = $(el).children('span');
            return <Chapter>{
                sourceId: this.id,
                title: $(chapterName).text(),
                path: $(chapterName).attr('href'),
                number: parseFloat($(chapterName).text().replace('Chapter ', '')),
                uploaded: moment($(chapterTime).text(), 'DD MMM YYYY').unix(),
            }
        });

        return Promise.resolve(chapters);
    }
    async getPages(path: string): Promise<string[]> {
        let body = await fetch(`${this.url}${path}`).then(res => res.text());
        const $ = cheerio.load(body);
        let pages = $('.read-content > img').toArray().map(el => $(el).attr('src')!);
        return Promise.resolve(pages);
    }

}