import Source from '../src/Guya';

const s = new Source();

export async function testGetLatestManga() {
    let manga = await s.getLatestManga(1);
    if (manga.length !== 5) {
        throw new Error("manga is not 5");
    }
}


export async function testGetPopularManga() {
    let manga = await s.getPopularManga(1);
    if (manga.length !== 5) {
        throw new Error("manga is not 5");
    }
}

export async function testSearchManga() {
    let manga = await s.searchManga(1, 'kaguya');
}

export async function testGetMangaDetail() {
    let manga = await s.getMangaDetail("/api/series/Kaguya-Wants-To-Be-Confessed-To");

    if (manga.title !== 'Kaguya-sama: Love is War') {
        throw new Error(`manga title is not 'Kaguya-sama: Love is War'`)
    }
}

export async function testGetChapters() {
    var chapters = await s.getChapters("/api/series/Kaguya-Wants-To-Be-Confessed-To");
}

export async function testGetPages() {
    var pages = await s.getPages("/api/series/Kaguya-Wants-To-Be-Confessed-To/3");
}