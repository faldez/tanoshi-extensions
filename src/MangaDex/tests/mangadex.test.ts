import { Group, State, Text, TriState } from 'tanoshi-extension-lib';
import Source from '../src';

const s = new Source();

export async function testGetLatestManga() {
    let manga = await s.getLatestManga(1);
    if (manga.length !== 20) {
        throw new Error(`expect manga.length 20 got ${manga.length}`);
    }
}

export async function testGetFilters() {
    let filters = s.filterList();
}

export async function testGetPopularManga() {
    let manga = await s.getPopularManga(1);
    if (manga.length !== 20) {
        throw new Error(`expect manga.length 20 got ${manga.length}`);
    }
}

export async function testSearchManga() {
    let manga = await s.searchManga(1, 'piece');
}

export async function testSearchMangaWithFilter() {
    let filters = s.filterList();
    let manga = await s.searchManga(1, undefined, filters);
}

export async function testGetMangaDetail() {
    let manga = await s.getMangaDetail("/manga/a96676e5-8ae2-425e-b549-7f15dd34a6d8");

    if (manga.title !== 'Komi-san wa Komyushou Desu.') {
        throw new Error(`expect Komi-san wa Komyushou Desu. got ${manga.title}`)
    }
}

export async function testGetChapters() {
    var chapters = await s.getChapters("/manga/a96676e5-8ae2-425e-b549-7f15dd34a6d8");
}

export async function testGetPages() {
    var pages = await s.getPages("/chapter/d35c7f27-9ad7-43d1-afb8-445dab0cb44e");
}