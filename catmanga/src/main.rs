mod data;

pub static ID: i64 = 5;
pub static NAME: &str = "catmanga";
pub static URL: &str = "https://catmanga.org";

use chrono::Local;
use data::SingleRoot;
use scraper::{Html, Selector};
use tanoshi_lib::prelude::*;
use tanoshi_util::*;

use crate::data::Root;

struct Catmanga;

impl Default for Catmanga {
    fn default() -> Self {
        Self {}
    }
}

register_extension!(Catmanga);

impl Catmanga {
    fn get_data() -> Option<Root> {
        let resp = http_request(Request {
            method: "GET".to_string(),
            url: URL.to_string(),
            headers: None,
        });
        if resp.status > 299 {
            return None;
        }
        let html = resp.body;
        let document = Html::parse_document(&html);
        let selector = Selector::parse("script[id=\"__NEXT_DATA__\"]").unwrap();

        let mut root: Option<Root> = None;
        if let Some(element) = document.select(&selector).next() {
            if let Some(text) = element.text().next() {
                root = serde_json::from_str(text).unwrap();
            }
        }

        root
    }

    fn get_single_data(path: String) -> Option<SingleRoot> {
        let resp = http_request(Request {
            method: "GET".to_string(),
            url: format!("{}{}", URL, path),
            headers: None,
        });
        if resp.status > 299 {
            return None;
        }
        let html = resp.body;
        let document = Html::parse_document(&html);
        let selector = Selector::parse("script[id=\"__NEXT_DATA__\"]").unwrap();

        let mut root: Option<SingleRoot> = None;
        if let Some(element) = document.select(&selector).next() {
            if let Some(text) = element.text().next() {
                root = serde_json::from_str(text).unwrap();
            }
        }

        root
    }
}

impl Extension for Catmanga {
    fn detail(&self) -> Source {
        Source {
            id: ID,
            name: NAME.to_string(),
            url: URL.to_string(),
            version: std::env!("CARGO_PKG_VERSION").to_string(),
            icon: "https://catmanga.org/favicon.png".to_string(),
            need_login: false,
        }
    }

    fn get_manga_list(&self, _param: Param) -> ExtensionResult<Vec<Manga>> {
        let root = Self::get_data();

        let mut manga = vec![];
        if let Some(root) = root {
            for series in root.props.page_props.series {
                manga.push(Manga {
                    source_id: ID,
                    title: series.title,
                    author: series.authors,
                    genre: series.genres,
                    status: Some(series.status),
                    description: Some(series.description),
                    path: format!("/series/{}", series.series_id),
                    cover_url: series.cover_art.source,
                })
            }
        }

        ExtensionResult {
            data: Some(manga),
            error: None,
        }
    }

    fn get_manga_info(&self, path: String) -> ExtensionResult<Manga> {
        let param = Param {
            keyword: None,
            genres: None,
            page: None,
            sort_by: None,
            sort_order: None,
            auth: None,
        };

        let mut data = None;
        let mut error = None;
        let res = self.get_manga_list(param);
        if let Some(manga) = res.data {
            for m in manga {
                if m.path == path {
                    data = Some(m);
                    break;
                }
            }
        }

        if data.is_none() {
            error = Some(format!("manga not found"));
        }

        ExtensionResult { data, error }
    }

    fn get_chapters(&self, path: String) -> ExtensionResult<Vec<Chapter>> {
        let root = Self::get_single_data(path.clone());

        let mut data = None;
        let mut error = None;

        let dt = Local::now();
        if let Some(s) = root {
            let mut chapters = vec![];
            for chapter in s.props.page_props.series.chapters {
                chapters.push(Chapter {
                    source_id: ID,
                    title: format!("Chapter {} - {}", chapter.number, chapter.title.unwrap_or("".to_string()).clone()),
                    path: format!("{}/{}", path, chapter.number),
                    number: chapter.number,
                    scanlator: chapter.groups.get(0).unwrap_or(&"".to_string()).to_string(),
                    uploaded: dt.naive_local(),
                });
            }
            data = Some(chapters)
        }

        if data.is_none() {
            error = Some(format!("manga not found"));
        }

        ExtensionResult { data, error }
    }

    fn get_pages(&self, path: String) -> ExtensionResult<Vec<String>> {
        let root = Self::get_single_data(path);

        let mut data = None;
        let mut error = None;

        if let Some(root) = root {
            data = Some(root.props.page_props.pages);
        }

        if data.is_none() {
            error = Some(format!("manga not found"));
        }

        ExtensionResult {
            data,
            error,
        }
    }
}
