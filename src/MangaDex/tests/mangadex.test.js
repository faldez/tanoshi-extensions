import Source from '../../dist/MangaDex';

export async function main() {
    const s = new Source();

    var manga;

    // manga = await s.getLatestManga(1);
    // console.log(JSON.stringify(manga));
    // console.log(`\n`);

    // manga = await s.getLatestManga(2);
    // console.log(JSON.stringify(manga));
    // console.log(`\n`);

    manga = await s.getPopularManga(1);
    console.log(JSON.stringify(manga));
    console.log(`\n`);

    manga = await s.getPopularManga(2);
    console.log(JSON.stringify(manga));
    console.log(`\n`);

    // let filter = s.getFilterList();
    // console.log(JSON.stringify(filter));
    // console.log(`\n`);

    // filter = filter.filter((f) => f.name === "included tags").map((f) => {
    //     f.state = ["Oneshot"];
    //     return f
    // });

    // manga = await s.searchManga(1, undefined, filter);
    // console.log(JSON.stringify(manga));
    // console.log(`\n`);

    // manga = await s.searchManga(2, undefined, filter);
    // console.log(JSON.stringify(manga));
    // console.log(`\n`);

    // manga = await s.getMangaDetail("/manga/0c735bb0-1329-43ba-9c64-f27aa570ba3f");
    // console.log(JSON.stringify(manga));
    // console.log(`\n`);

    // var chapters = await s.getChapters("/manga/77bee52c-d2d6-44ad-a33a-1734c1fe696a");
    // console.log(JSON.stringify(chapters));
    // console.log(`\n`);

    // var pages = await s.getPages("/chapter/fd2c99d6-8b41-4aca-acd0-f954115d881c");
    // console.log(JSON.stringify(pages));
    // console.log(`\n`);
}