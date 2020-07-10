use std::collections::HashMap;

use anyhow::Result;
use bimap::BiMap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_urlencoded;
use tanoshi_lib::extensions::Extension;
use ureq;

lazy_static! {
    static ref STATUS: BiMap<i64, &'static str> = {
        let mut m = BiMap::new();
        m.insert(1, "Ongoing");
        m.insert(2, "Completed");
        m.insert(3, "Cancelled");
        m.insert(4, "Hiatus");
        m
    };
    static ref GENRES: BiMap<i64, &'static str> = {
        let mut m = BiMap::new();
        m.insert(9, "Ecchi");
        m.insert(49, "Gore");
        m.insert(50, "Sexual Violence");
        m.insert(32, "Smut");
        m.insert(1, "4-Koma");
        m.insert(42, "Adaptation");
        m.insert(43, "Anthology");
        m.insert(4, "Award Winning");
        m.insert(7, "Doujinshi");
        m.insert(48, "Fan Colored");
        m.insert(45, "Full Color");
        m.insert(36, "Long Strip");
        m.insert(47, "Official Colored");
        m.insert(21, "Oneshot");
        m.insert(46, "User Created");
        m.insert(44, "Web Comic");
        m.insert(2, "Action");
        m.insert(3, "Adventure");
        m.insert(5, "Comedy");
        m.insert(51, "Crime");
        m.insert(8, "Drama");
        m.insert(10, "Fantasy");
        m.insert(13, "Historical");
        m.insert(14, "Horror");
        m.insert(41, "Isekai");
        m.insert(52, "Magical Girls");
        m.insert(17, "Mecha");
        m.insert(18, "Medical");
        m.insert(20, "Mystery");
        m.insert(53, "Philosophical");
        m.insert(22, "Psychological");
        m.insert(23, "Romance");
        m.insert(25, "Sci-Fi");
        m.insert(28, "Shoujo Ai");
        m.insert(30, "Shounen Ai");
        m.insert(31, "Slice of Life");
        m.insert(33, "Sports");
        m.insert(54, "Superhero");
        m.insert(55, "Thriller");
        m.insert(35, "Tragedy");
        m.insert(56, "Wuxia");
        m.insert(37, "Yaoi");
        m.insert(38, "Yuri");
        m.insert(57, "Aliens");
        m.insert(58, "Animals");
        m.insert(6, "Cooking");
        m.insert(59, "Crossdressing");
        m.insert(61, "Delinquents");
        m.insert(60, "Demons");
        m.insert(62, "Genderswap");
        m.insert(63, "Ghosts");
        m.insert(11, "Gyaru");
        m.insert(12, "Harem");
        m.insert(83, "Incest");
        m.insert(65, "Loli");
        m.insert(84, "Mafia");
        m.insert(66, "Magic");
        m.insert(16, "Martial Arts");
        m.insert(67, "Military");
        m.insert(64, "Monster Girls");
        m.insert(68, "Monsters");
        m.insert(19, "Music");
        m.insert(69, "Ninja");
        m.insert(70, "Office Workers");
        m.insert(71, "Police");
        m.insert(72, "Post-Apocalyptic");
        m.insert(73, "Reincarnation");
        m.insert(74, "Reverse Harem");
        m.insert(75, "Samurai");
        m.insert(24, "School Life");
        m.insert(76, "Shota");
        m.insert(34, "Supernatural");
        m.insert(77, "Survival");
        m.insert(78, "Time Travel");
        m.insert(80, "Traditional Games");
        m.insert(79, "Vampires");
        m.insert(40, "Video Games");
        m.insert(81, "Virtual Reality");
        m.insert(82, "Zombies");
        m
    };
}

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
    pub chapter: HashMap<String, Chapter>,
    pub status: String,
}

impl Into<Vec<tanoshi_lib::manga::Chapter>> for GetMangaResponse {
    fn into(self) -> Vec<tanoshi_lib::manga::Chapter> {
        self.chapter
            .par_iter()
            .map(|(id, chapter)| {
                if chapter.lang_code == "gb".to_string() {
                    Some(tanoshi_lib::manga::Chapter {
                        id: 0,
                        manga_id: 0,
                        vol: if chapter.volume == "" {
                            None
                        } else {
                            Some(chapter.volume.clone())
                        },
                        no: if chapter.chapter == "" {
                            None
                        } else {
                            Some(chapter.chapter.clone())
                        },
                        title: if chapter.title == "" {
                            None
                        } else {
                            Some(chapter.title.clone())
                        },
                        url: format!("/api/chapter/{}", id),
                        read: None,
                        uploaded: chrono::NaiveDateTime::from_timestamp(chapter.timestamp, 0),
                    })
                } else {
                    None
                }
            })
            .filter_map(|ch| ch)
            .collect()
    }
}

impl Into<tanoshi_lib::manga::Manga> for GetMangaResponse {
    fn into(self) -> tanoshi_lib::manga::Manga {
        let description_split = self.manga.description.split("\r\n").collect::<Vec<_>>();
        let description = match description_split[0].to_string().starts_with("[b][u]") {
            true => description_split[1].to_string(),
            false => description_split[0].to_string(),
        };
        tanoshi_lib::manga::Manga {
            id: 0,
            title: self.manga.title.into(),
            author: vec![self.manga.author, self.manga.artist],
            genre: self
                .manga
                .genres
                .par_iter()
                .map(|genre| GENRES.get_by_left(genre).unwrap().to_string())
                .collect(),
            status: STATUS
                .get_by_left(&self.manga.status)
                .map(|s| s.to_string()),
            description: Some(description),
            path: "".to_string(),
            thumbnail_url: format!("https://mangadex.org{}", self.manga.cover_url),
            last_read: None,
            last_page: None,
            is_favorite: false,
        }
    }
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

impl Into<Vec<String>> for GetPagesResponse {
    fn into(self) -> Vec<String> {
        let host = self.server;
        let hash = self.hash;
        self.page_array
            .par_iter()
            .map(|p| format!("{}{}/{}", host, hash, p))
            .collect()
    }
}

pub struct Mangadex {}

impl Extension for Mangadex {
    fn info(&self) -> tanoshi_lib::manga::Source {
        tanoshi_lib::manga::Source {
            id: 0,
            name: "mangadex".to_string(),
            url: "https://mangadex.org".to_string(),
            version: std::env!("PLUGIN_VERSION").to_string(),
        }
    }

    fn get_mangas(
        &self,
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
            ("tags".to_owned(), param.genres.map(|t| t.join(","))),
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

    fn get_manga_info(&self, url: &String) -> Result<tanoshi_lib::manga::Manga> {
        let resp = ureq::get(url.as_str()).call();
        let mangadex_resp = resp.into_json_deserialize::<GetMangaResponse>().unwrap();

        Ok(mangadex_resp.into())
    }

    fn get_chapters(&self, url: &String) -> Result<Vec<tanoshi_lib::manga::Chapter>> {
        let resp = ureq::get(url.as_str()).call();
        let mangadex_resp = resp.into_json_deserialize::<GetMangaResponse>().unwrap();

        Ok(mangadex_resp.into())
    }

    fn get_pages(&self, url: &String) -> Result<Vec<String>> {
        let resp = ureq::get(url.as_str()).call();
        let mangadex_resp = resp.into_json_deserialize::<GetPagesResponse>().unwrap();

        Ok(mangadex_resp.into())
    }

    fn login(
        &self,
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
