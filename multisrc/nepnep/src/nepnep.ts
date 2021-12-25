import * as cheerio from 'cheerio';
import { Chapter, Extension, Input, Text, Group, Manga, fetch, State, Sort, Select, TriState } from "tanoshi-extension-lib";
import * as moment from 'moment';

import { Genre as GenreList, SortBy, ScanStatus, PublishStatus } from './filters.json';
import { Directory } from './dto';

export abstract class NepNep extends Extension {
    protected keywordFilter = new Text("Series Name", "");
    protected genreFilter = new Group("Genres", GenreList.map((g) => new State(g)));
    protected scanStatusFilter = new Select("Scan Status", ["Any", ...ScanStatus]);
    protected publishStatusFilter = new Select("Publish Status", ["Any", ...PublishStatus]);
    protected sortByFilter = new Sort("Sort By", SortBy);

    override getFilterList(): Input[] {
        return [
            this.keywordFilter,
            this.scanStatusFilter,
            this.publishStatusFilter,
            this.sortByFilter,
            this.genreFilter
        ];
    }

    protected async getAllManga(): Promise<Directory[]> {
        var body = await fetch(`${this.url}/search`).then(res => res.text());

        body = body.substring(body.search("vm.Directory = ") + 15);
        body = body.substring(0, body.search("];") + 1);

        var data = JSON.parse(body);

        return data;
    }

    protected mapDataToManga(data: Directory[], page: number): Manga[] {
        if (page < 1) {
            page = 1;
        }

        var offset = (page - 1) * 20;
        data = data.slice(offset, offset + 20);

        return data.map(item => <Manga>{
            sourceId: this.id,
            title: item.s,
            author: item.a,
            status: item.ps,
            genre: item.g,
            path: `/manga/${item['i']}`,
            coverUrl: `https://cover.nep.li/cover/${item.i}.jpg`,
        });
    }

    override async getPopularManga(page: number): Promise<Manga[]> {
        var data = await this.getAllManga();

        var data = data.sort((a, b) => {
            if (a.v > b.v) {
                return -1;
            }

            if (a.v < b.v) {
                return 1;
            }

            return 0;
        });

        return Promise.resolve(this.mapDataToManga(data, page));
    }

    override async getLatestManga(page: number): Promise<Manga[]> {
        var data = await this.getAllManga();

        var data = data.sort((a, b) => {
            if (a.lt > b.lt) {
                return -1;
            }

            if (a.lt < b.lt) {
                return 1;
            }

            return 0;
        });

        return Promise.resolve(this.mapDataToManga(data, page));
    }

    protected filterKeyword(data: Directory[], state: string): Directory[] {
        return data.filter((item) => {
            return (item.s.toLowerCase().indexOf(state) != -1);
        });
    }

    protected filterIncludeGenres(data: Directory[], input: Group<State>): Directory[] {
        let state = input.state;
        let includedGenre = state?.filter((genre) => genre.selected == TriState.Included).map(g => g.name);
        if (includedGenre?.length! > 0) {
            data = data.filter((item) => {
                let set = new Set([...item.g]);
                for (const genre of includedGenre!) {
                    if (set.has(genre)) {
                        return true;
                    }
                }
                return false;
            });
        }

        let excludedGenre = state?.filter((genre) => genre.selected == TriState.Excluded).map(g => g.name);
        if (excludedGenre?.length! > 0) {
            data = data.filter((item) => {
                let set = new Set([...item.g]);
                for (const genre of excludedGenre!) {
                    if (set.has(genre)) {
                        return true;
                    }
                }
                return true;
            });
        }

        return data;
    }

    override async searchManga(page: number, query?: string, filter?: Input[]): Promise<Manga[]> {
        if (query === undefined && filter === undefined) {
            throw new Error("query and filters cannot be both empty");
        }

        var data = await this.getAllManga();

        if (filter) {
            for (var input of filter) {
                if (this.keywordFilter.equals(input) && (input as Text).state! != '') {
                    data = this.filterKeyword(data, (input as Text).state!);
                }

                if (this.genreFilter.equals(input)) {
                    data = this.filterIncludeGenres(data, input);
                }
            }
        } else if (query) {
            data = this.filterKeyword(data, query);
        }


        var manga = this.mapDataToManga(data, page);

        return Promise.resolve(manga);
    }

    override async getMangaDetail(path: string): Promise<Manga> {
        var body = await fetch(`${this.url}${path}`).then(res => res.text());
        const $ = cheerio.load(body);

        const title = $('li[class=\"list-group-item d-none d-sm-block\"] h1').text();
        const description = $('div[class=\"top-5 Content\"]').text();
        const authors = $('a[href^=\"/search/?author=\"]').toArray().map((el) => $(el).text());
        const genres = $('a[href^=\"/search/?genre=\"]').toArray().map((el) => $(el).text());
        const status = $('a[href^=\"/search/?status=\"]').first().attr("href")?.replace("/search/?status=", "");
        const coverUrl = $('img[class=\"img-fluid bottom-5\"]').attr("src");

        return Promise.resolve({
            sourceId: this.id,
            title: title,
            author: authors,
            status: status,
            description: description,
            genre: genres,
            path: `${path}`,
            coverUrl: coverUrl!,
        });
    }

    chapterDisplay(e: string): number {
        var t = e.slice(1, -1), n = e[e.length - 1];
        return parseFloat(`${t}.${n}`);
    }

    override async getChapters(path: string): Promise<Chapter[]> {
        var body = await fetch(`${this.url}${path}`).then(res => res.text());
        if (!body) {
            return Promise.reject(`failed to fetch chapters for ${path}`);
        }

        let matchIndexName = body.match(/(?<=vm\.IndexName = ").*(?=";)/g);
        if (!matchIndexName) {
            return Promise.reject(`indexName not found for ${path}`);
        }
        var indexName = matchIndexName[0];
        let matchChapters = body.match(/(?<=vm\.Chapters = )\[.*\](?=;)/g);
        if (!matchChapters) {
            return Promise.reject(`chapters not found for ${path}`);
        }
        var chapters = JSON.parse(matchChapters[0]);

        return Promise.resolve(chapters.map((item: any) => {
            let number = this.chapterDisplay(item['Chapter']);
            var ch: Chapter = {
                sourceId: this.id,
                title: `${item['Type']} ${number}`,
                path: `/read-online/${indexName}-chapter-${number}${item['Chapter'][0] != '1' ? '-index-' + item['Chapter'][0] : ''}.html`,
                number: number,
                uploaded: moment(item['Date'], moment.ISO_8601).unix(),
            };
            return ch;
        }));
    }

    chapterImage = (chapterString: string) => {
        var Chapter = chapterString.slice(1, -1);
        var Odd = chapterString[chapterString.length - 1];
        if (Odd == '0') {
            return Chapter;
        } else {
            return Chapter + "." + Odd;
        }
    };
    pageImage = (pageString: string) => {
        var s = "000" + pageString;
        return s.substring(s.length - 3);
    }

    override async getPages(path: string): Promise<string[]> {
        var body = await fetch(`${this.url}${path}`).then(res => res.text());
        var curPathName = body.match(/(?<=vm\.CurPathName = ").*(?=";)/g)![0];
        var curChapter = JSON.parse(body.match(/(?<=vm\.CurChapter = ).*(?=;)/g)![0]);
        var indexName = body.match(/(?<=vm\.IndexName = ").*(?=";)/g)![0];

        let pages = [];
        for (let i = 1; i <= parseInt(curChapter['Page']); i++) {
            pages.push(`https://${curPathName}/manga/${indexName}/${curChapter['Directory'] == '' ? '' : curChapter.Directory + '/'}${this.chapterImage(curChapter['Chapter'])}-${this.pageImage(i.toString())}.png`);
        }

        return Promise.resolve(pages);
    }
}
