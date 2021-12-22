import { print } from 'tanoshi-extension-lib';
import Source from '../src';

const s = new Source();

export async function testGetLatestManga() {
    let manga = await s.getLatestManga(1);
    if (manga.length !== 25) {
        throw new Error("manga is not 25");
    }
}


export async function testGetPopularManga() {
    let manga = await s.getPopularManga(1);
    if (manga.length !== 25) {
        throw new Error("manga is not 25");
    }
}

export async function testSearchManga() {
    let manga = await s.searchManga(1, 'kaguya');
}

export async function testGetMangaDetail() {
    let manga = await s.getMangaDetail("/api/gallery/384090");

    if (manga.title !== 'Boku no Osananajimi Again | My Childhood Friend Again') {
        throw new Error(`expect Boku no Osananajimi Again | My Childhood Friend Again got ${manga.title}`)
    }
}

export async function testGetChapters() {
    var chapters = await s.getChapters("/api/gallery/384090");
}

export async function testGetPages() {
    var pages = await s.getPages("/api/gallery/384090");
}