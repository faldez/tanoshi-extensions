import Source from '../src/Manhwa18cc';

const s = new Source();

export async function testGetLatestManga() {
    let manga = await s.getLatestManga(1);
    if (manga.length !== 24) {
        throw new Error("manga is not 24");
    }
}


export async function testGetPopularManga() {
    let manga = await s.getPopularManga(1);
    if (manga.length !== 24) {
        throw new Error("manga is not 24");
    }
}

export async function testSearchManga() {
    let manga = await s.searchManga(1, 'private');
}

export async function testGetMangaDetail() {
    let manga = await s.getMangaDetail("/webtoon/private-tutoring-in-these-trying-times");

    if (manga.title !== 'Private Tutoring in These Trying Times') {
        throw new Error(`expect Private Tutoring in These Trying Times got ${manga.title}`)
    }
}

export async function testGetChapters() {
    var chapters = await s.getChapters("/webtoon/private-tutoring-in-these-trying-times");
}

export async function testGetPages() {
    var pages = await s.getPages("/webtoon/private-tutoring-in-these-trying-times/chapter-27");
}