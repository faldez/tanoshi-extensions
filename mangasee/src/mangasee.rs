use anyhow::Result;
use serde_urlencoded;
use tanoshi::scraping::Scraping;
use tanoshi::manga::{
    Chapter, Manga, Params, SortOrderParam, SortByParam
};

#[no_mangle]
pub extern "C" fn get_mangas(url: &String, param: Params, cookies: Vec<String>) -> Result<Vec<Manga>> {
    Mangasee::get_mangas(url, param, cookies)
}

#[no_mangle]
pub extern "C" fn get_manga_info(url: &String) -> Result<Manga> {
    Mangasee::get_manga_info(url)
}

#[no_mangle]
pub extern "C" fn get_chapters(url: &String) -> Result<Vec<Chapter>> {
    Mangasee::get_chapters(url)
}

#[no_mangle]
pub extern "C" fn get_pages(url: &String) -> Result<Vec<String>> {
    Mangasee::get_pages(url)
}

pub struct Mangasee {}

impl Scraping for Mangasee {
    fn get_mangas(url: &String, param: Params, _: Vec<String>) -> Result<Vec<Manga>> {
        let mut mangas: Vec<Manga> = Vec::new();

        let sort_by = match param.sort_by.unwrap() {
            SortByParam::Views => "popularity",
            SortByParam::LastUpdated => "dateUpdated",
            _ => "dateUpdated"
        };

        let sort_order = match param.sort_order.unwrap() {
            SortOrderParam::Asc => "ascending",
            SortOrderParam::Desc => "descending"
        };

        let params = vec![
            ("keyword".to_owned(), param.keyword.to_owned()),
            ("page".to_owned(), param.page.to_owned()),
            ("sortBy".to_owned(), Some(sort_by.to_string())),
            ("sortOrder".to_owned(), Some(sort_order.to_string())),
        ];

        let urlencoded = serde_urlencoded::to_string(params).unwrap();

        let resp = ureq::post(format!("{}/search/request.php", url).as_str())
            .set(
                "Content-Type",
                "application/x-www-form-urlencoded; charset=utf-8",
            )
            .send_string(&urlencoded);

        let html = resp.into_string().unwrap();
        let document = scraper::Html::parse_document(&html);

        let selector = scraper::Selector::parse(".requested .row").unwrap();
        for row in document.select(&selector) {
            let mut manga = Manga::default();

            let sel = scraper::Selector::parse("img").unwrap();
            for el in row.select(&sel) {
                manga.thumbnail_url = el.value().attr("src").unwrap().to_owned();
            }

            let sel = scraper::Selector::parse(".resultLink").unwrap();
            for el in row.select(&sel) {
                manga.title = el.inner_html();
                manga.path = el.value().attr("href").unwrap().to_owned();
            }
            mangas.push(manga);
        }

        Ok(mangas)
    }

    fn get_manga_info(url: &String) -> Result<Manga> {
        let mut m = Manga::default();

        let resp = ureq::get(url.as_str()).call();
        let html = resp.into_string().unwrap();

        let document = scraper::Html::parse_document(&html);

        let selector = scraper::Selector::parse(".leftImage img").unwrap();
        for element in document.select(&selector) {
            let src = element.value().attr("src").unwrap();
            m.thumbnail_url = String::from(src);
        }

        let selector = scraper::Selector::parse("h1[class=\"SeriesName\"]").unwrap();
        for element in document.select(&selector) {
            m.title = element.inner_html();
        }

        let selector = scraper::Selector::parse("a[href*=\"author\"]").unwrap();

        for element in document.select(&selector) {
            for text in element.text() {
                m.author = String::from(text);
            }
        }

        /* let selector = scraper::Selector::parse("a[href*=\"genre\"]").unwrap();
        for element in document.select(&selector) {
            for text in element.text() {
                m.genre.push(String::from(text));
            }
        } */

        let selector = scraper::Selector::parse(".PublishStatus").unwrap();
        for element in document.select(&selector) {
            let status = element.value().attr("status").unwrap();
            m.status = String::from(status);
        }

        let selector = scraper::Selector::parse(".description").unwrap();
        for element in document.select(&selector) {
            for text in element.text() {
                m.description = String::from(text);
            }
        }

        Ok(m)
    }

    fn get_chapters(url: &String) -> Result<Vec<Chapter>> {
        let mut chapters: Vec<Chapter> = Vec::new();
        let resp = ureq::get(url.as_str()).call();
        let html = resp.into_string().unwrap();

        let document = scraper::Html::parse_document(&html);
        let selector = scraper::Selector::parse(".mainWell .chapter-list a[chapter]").unwrap();
        for element in document.select(&selector) {
            let mut chapter = Chapter::default();

            chapter.no = String::from(element.value().attr("chapter").unwrap());

            let link = element.value().attr("href").unwrap();
            chapter.url = link.replace("-page-1", "");

            let time_sel = scraper::Selector::parse("time[class*=\"SeriesTime\"]").unwrap();
            for time_el in element.select(&time_sel) {
                let date_str = time_el.value().attr("datetime").unwrap();
                chapter.uploaded = chrono::NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S%:z").unwrap()
            }

            chapters.push(chapter);
        }

        Ok(chapters)
    }

    fn get_pages(url: &String) -> Result<Vec<String>> {
        let mut pages = Vec::new();
        let resp = ureq::get(url.as_str()).call();
        let html = resp.into_string().unwrap();

        let document = scraper::Html::parse_document(&html);

        let selector = scraper::Selector::parse(".fullchapimage img").unwrap();
        for element in document.select(&selector) {
            pages.push(String::from(element.value().attr("src").unwrap()));
        }
        Ok(pages)
    }
}
