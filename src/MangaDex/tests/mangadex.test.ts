import Source from '../src';

const s = new Source();

export async function testGetLatestManga() {
    let manga = await s.getLatestManga(1);
    if (manga.length !== 20) {
        throw new Error(`expect manga.length 20 got ${manga.length}`);
    }
}

export async function testGetFilters() {
    let filters = s.getFilterList();
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
    let filters = s.getFilterList();
    let manga = await s.searchManga(1, undefined, filters);
}

export async function testGetMangaDetail() {
    let manga = await s.getMangaDetail("/manga/One-Piece");

    if (manga.title !== 'One Piece') {
        throw new Error(`expect One Piece got ${manga.title}`)
    }
}

export async function testGetChapters() {
    var chapters = await s.getChapters("/manga/One-Piece");
}

export async function testGetPages() {
    var pages = await s.getPages("/read-online/One-Piece-chapter-1035-page-1.html");
}