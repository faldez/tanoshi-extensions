import Source from '../../dist/MangaSee';

export async function main() {
    const s = new Source();
    console.log(JSON.stringify(s));

    let manga = await s.getPopularManga(1);
    console.log(JSON.stringify(manga));
}