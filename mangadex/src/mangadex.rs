use anyhow::Result;
use serde_urlencoded;
use ureq;

use tanoshi_lib::extensions::Extension;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct MangadexLogin {
    pub login_username: String,
    pub login_password: String,
    pub remember_me: bool,
    pub two_factor: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetMangaResponse {
    pub manga: Manga,
    pub status: String,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetChapterResponse {
    pub chapter: HashMap<String, Chapter>,
    pub status: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manga {
    pub cover_url: String,
    pub description: String,
    pub title: String,
    pub artist: String,
    pub author: String,
    pub status: i64,
    pub genres: Vec<i64>,
    pub last_chapter: String,
    pub lang_name: String,
    pub lang_flag: String,
    pub hentai: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    pub volume: String,
    pub chapter: String,
    pub title: String,
    pub lang_code: String,
    pub timestamp: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetPagesResponse {
    pub id: i64,
    pub timestamp: i64,
    pub hash: String,
    pub volume: String,
    pub chapter: String,
    pub title: String,
    pub server: String,
    pub page_array: Vec<String>,
    pub status: String,
}

pub struct Mangadex {}

impl Extension for Mangadex {
    fn info(&mut self) -> tanoshi_lib::manga::Source {
        tanoshi_lib::manga::Source {
            id: 0,
            name: "mangadex".to_string(),
            url: "https://mangadex.org".to_string(),
            version: std::env!("PLUGIN_VERSION").to_string(),
        }
    }

    fn get_mangas(
        &mut self,
        url: &String,
        param: tanoshi_lib::manga::Params,
        auth: String,
    ) -> Result<Vec<tanoshi_lib::manga::Manga>> {
        let mut mangas: Vec<tanoshi_lib::manga::Manga> = Vec::new();

        let mut s = match param.sort_by.unwrap() {
            tanoshi_lib::manga::SortByParam::LastUpdated => 0,
            tanoshi_lib::manga::SortByParam::Views => 8,
            tanoshi_lib::manga::SortByParam::Title => 2,
            _ => 0,
        };

        s = match param.sort_order.unwrap() {
            tanoshi_lib::manga::SortOrderParam::Asc => s,
            tanoshi_lib::manga::SortOrderParam::Desc => s + 1,
        };

        let params = vec![
            ("title".to_owned(), param.keyword.to_owned()),
            ("p".to_owned(), param.page.to_owned()),
            ("s".to_owned(), Some(s.to_string())),
        ];

        let urlencoded = serde_urlencoded::to_string(params).unwrap();

        let resp = ureq::get(format!("{}/search?{}", url.clone(), urlencoded).as_str())
            .set("Cookie", &auth)
            .call();

        let html = resp.into_string().unwrap();
        let document = scraper::Html::parse_document(&html);

        let selector = scraper::Selector::parse(".manga-entry").unwrap();
        for row in document.select(&selector) {
            let mut manga = tanoshi_lib::manga::Manga::default();
            let id = row.value().attr("data-id").unwrap();
            manga.path = format!("/api/manga/{}", id);

            let sel = scraper::Selector::parse("div a img").unwrap();
            for el in row.select(&sel) {
                manga.thumbnail_url =
                    format!("{}{}", url, el.value().attr("src").unwrap().to_owned());
            }

            let sel = scraper::Selector::parse(".manga_title").unwrap();
            for el in row.select(&sel) {
                manga.title = el.inner_html();
            }
            mangas.push(manga);
        }

        Ok(mangas)
    }

    fn get_manga_info(&mut self, url: &String) -> Result<tanoshi_lib::manga::Manga> {
        let resp = ureq::get(url.as_str()).call();
        let mangadex_resp: GetMangaResponse = serde_json::from_reader(resp.into_reader()).unwrap();

        let description_split = mangadex_resp
            .manga
            .description
            .split("\r\n")
            .collect::<Vec<_>>();
        let description = match description_split[0].to_string().starts_with("[b][u]") {
            true => description_split[1].to_string(),
            false => description_split[0].to_string(),
        };
        let m = tanoshi_lib::manga::Manga {
            id: 0,
            title: mangadex_resp.manga.title,
            author: mangadex_resp.manga.author,
            //genre: vec![],
            status: match mangadex_resp.manga.status {
                1 => "Ongoing".to_string(),
                2 => "Completed".to_string(),
                3 => "Cancelled".to_string(),
                4 => "Hiatus".to_string(),
                _ => "Ongoing".to_string(),
            },
            description,
            path: "".to_string(),
            thumbnail_url: format!("https://mangadex.org{}", mangadex_resp.manga.cover_url),
            last_read: None,
            last_page: None,
            is_favorite: false,
        };

        Ok(m)
    }

    fn get_chapters(&mut self, url: &String) -> Result<Vec<tanoshi_lib::manga::Chapter>> {
        let mut chapters: Vec<tanoshi_lib::manga::Chapter> = Vec::new();

        let resp = ureq::get(url.as_str()).call();
        let mangadex_resp: GetChapterResponse =
            serde_json::from_reader(resp.into_reader()).unwrap();

        for (id, chapter) in mangadex_resp.chapter {
            if chapter.lang_code == "gb".to_string() {
                chapters.push(tanoshi_lib::manga::Chapter {
                    id: 0,
                    manga_id: 0,
                    no: match chapter.chapter.as_str() {
                        "" => "0".to_string(),
                        _ => chapter.chapter,
                    },
                    title: chapter.title,
                    url: format!("/api/chapter/{}", id),
                    read: 0,
                    uploaded: chrono::NaiveDateTime::from_timestamp(chapter.timestamp, 0),
                })
            }
        }

        Ok(chapters)
    }

    fn get_pages(&mut self, url: &String) -> Result<Vec<String>> {
        let mut pages = Vec::new();

        let resp = ureq::get(url.as_str()).call();
        let mangadex_resp: GetPagesResponse = serde_json::from_reader(resp.into_reader()).unwrap();

        for page in mangadex_resp.page_array {
            pages.push(format!(
                "{}{}/{}",
                mangadex_resp.server, mangadex_resp.hash, page
            ));
        }

        Ok(pages)
    }

    fn login(
        &mut self,
        login: tanoshi_lib::manga::SourceLogin,
    ) -> Result<tanoshi_lib::manga::SourceLoginResult> {
        let boundary = "__TANOSHI__";
        let mut param = vec![];
        param.push(format!(
            "--{}\nContent-Disposition: form-data; name=\"login_username\"\n\n{}",
            boundary, login.username
        ));
        param.push(format!(
            "--{}\nContent-Disposition: form-data; name=\"login_password\"\n\n{}",
            boundary, login.password
        ));
        if let Some(remember_me) = login.remember_me {
            param.push(format!(
                "--{}\nContent-Disposition: form-data; name=\"remember_me\"\n\n{}",
                boundary, remember_me as i32
            ));
        }
        if let Some(two_factor) = login.two_factor {
            param.push(format!(
                "--{}\nContent-Disposition: form-data; name=\"two_factor\"\n\n{}",
                boundary, two_factor
            ));
        }
        param.push(format!("--{}--", boundary));

        let resp = ureq::post("https://mangadex.org/ajax/actions.ajax.php?function=login")
            .set("X-Requested-With", "XMLHttpRequest")
            .set(
                "Content-Type",
                format!("multipart/form-data; charset=utf-8; boundary={}", boundary).as_str(),
            )
            .set("User-Agent", "Tanoshi/0.1.0")
            .send_string(&param.join("\n"));

        let cookies = resp
            .all("Set-Cookie")
            .into_iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>();

        Ok(tanoshi_lib::manga::SourceLoginResult {
            source_id: 0,
            source_name: "mangadex".to_string(),
            auth_type: "cookies".to_string(),
            value: cookies.join("; "),
        })
    }
}
