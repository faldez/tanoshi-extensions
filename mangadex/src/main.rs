use chrono::NaiveDateTime;
use data::Result;
use fancy_regex::Regex;
use tanoshi_lib::prelude::*;
use tanoshi_util::*;

use crate::data::{
    manga::{request, ListOrder, Order},
    Home, Results,
};

mod data;

pub static ID: i64 = 2;
pub static NAME: &str = "mangadex";
pub static URL: &str = "https://api.mangadex.org";

pub struct Mangadex {
    url: String,
}

register_extension!(Mangadex);

impl Default for Mangadex {
    fn default() -> Self {
        Mangadex {
            url: URL.to_string(),
        }
    }
}

impl Mangadex {
    #[must_use]
    fn remove_bbcode(string: &str) -> String {
        let regex = Regex::new(r#"\[(\w+)[^]]*](.*?)\[/\1]"#).unwrap();

        let result = string
            .replace("[list]", "")
            .replace("[/list]", "")
            .replace("[*]", "")
            .replace("[hr]", "\n");

        regex.replace_all(&result, "$2").to_string()
    }

    pub fn map_result_to_manga(result: Result) -> Option<Manga> {
        let mut author = vec![];
        let mut genre = vec![];
        let mut file_name = "".to_string();
        for relationship in result.relationships {
            match relationship {
                data::Relationship::Author { attributes, .. } => {
                    if let Some(name) = attributes.map(|attr| attr.name) {
                        author.push(name);
                    }
                }
                data::Relationship::Artist { attributes, .. } => {
                    if let Some(name) = attributes.map(|attr| attr.name) {
                        author.push(name);
                    }
                }
                data::Relationship::Tag { attributes, .. } => {
                    if let Some(name) = attributes.and_then(|attr| attr.name.get("en").cloned()) {
                        genre.push(name.to_owned());
                    }
                }
                data::Relationship::CoverArt { attributes, .. } => {
                    if let Some(name) = attributes.map(|attr| attr.file_name) {
                        file_name = name;
                    }
                }
                _ => {}
            };
        }

        match result.data {
            data::Relationship::Manga { id, attributes } => Some(Manga {
                source_id: ID,
                title: attributes
                    .clone()
                    .and_then(|attr| attr.title.get("en").cloned())
                    .unwrap_or_else(|| "".to_string()),
                author,
                genre,
                status: attributes
                    .clone()
                    .and_then(|attr| attr.status.clone())
                    .map(|s| s.to_string()),
                description: attributes
                    .clone()
                    .and_then(|attr| attr.description.get("en").cloned())
                    .map(|description| Self::remove_bbcode(&description)),
                path: format!("/manga/{}", id),
                cover_url: format!("https://uploads.mangadex.org/covers/{}/{}", id, file_name),
            }),
            _ => None,
        }
    }

    pub fn map_result_to_chapter(result: Result) -> Option<Chapter> {
        let mut scanlator = "".to_string();
        for relationship in result.relationships {
            match relationship {
                data::Relationship::ScanlationGroup { attributes, .. } => {
                    if let Some(name) = attributes.map(|attr| attr.name) {
                        scanlator = name;
                    }
                }
                _ => {}
            };
        }

        match result.data {
            data::Relationship::Chapter { id, attributes } => {
                let volume = attributes.clone().and_then(|attr| attr.volume);
                let number = attributes.clone().and_then(|attr| attr.chapter);
                let mut title = attributes
                    .clone()
                    .and_then(|attr| attr.title)
                    .unwrap_or_else(|| "".to_string());

                if title.is_empty() {
                    if let Some(vol) = volume {
                        title = format!("Volume {}", vol);
                    }
                    if let Some(ch) = number.clone() {
                        title = format!("{} Chapter {}", title, ch)
                    }
                    title = title.trim().to_string();
                }

                Some(Chapter {
                    source_id: ID,
                    title,
                    path: format!("/chapter/{}", id),
                    number: number
                        .and_then(|chapter| chapter.parse().ok())
                        .unwrap_or_default(),
                    scanlator,
                    uploaded: attributes
                        .map(|attr| attr.publish_at.naive_utc())
                        .unwrap_or_else(|| NaiveDateTime::from_timestamp(0, 0)),
                })
            }
            _ => None,
        }
    }

    pub fn map_result_to_pages(result: Result) -> Option<Vec<String>> {
        match result.data {
            data::Relationship::Chapter { id, attributes } => {
                if let Some(attr) = attributes {
                    let req = Request {
                        method: "GET".to_string(),
                        url: format!("{}/at-home/server/{}", URL, id,),
                        headers: None,
                    };

                    let res = http_request(req);
                    let base_url = match serde_json::from_str::<Home>(&res.body) {
                        Ok(home) => home.base_url,
                        Err(_) => {
                            return None;
                        }
                    };

                    let pages = attr
                        .data
                        .iter()
                        .map(|page| format!("{}/data/{}/{}", base_url, attr.hash, page))
                        .collect();

                    Some(pages)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl Extension for Mangadex {
    fn detail(&self) -> Source {
        Source {
            id: ID,
            name: NAME.to_string(),
            url: self.url.clone(),
            version: std::env!("PLUGIN_VERSION").to_string(),
            icon: "https://api.mangadex.org/favicon.ico".to_string(),
            need_login: false,
            languages: vec!["en".to_string()],
        }
    }

    fn filters(&self) -> ExtensionResult<Option<Filters>> {
        ExtensionResult::ok(None)
    }

    fn get_manga_list(&self, param: Param) -> ExtensionResult<Vec<Manga>> {
        let order = match (param.sort_by, param.sort_order) {
            (Some(sort_by), Some(sort_order)) => {
                if matches!(sort_by, SortByParam::LastUpdated) {
                    let order = match sort_order {
                        SortOrderParam::Asc => Order::Asc,
                        SortOrderParam::Desc => Order::Desc,
                    };

                    Some(ListOrder {
                        created_at: None,
                        updated_at: Some(order),
                    })
                } else {
                    None
                }
            }
            _ => None,
        };

        let query = request::MangaList {
            includes: vec![
                "cover_art".to_string(),
                "author".to_string(),
                "artist".to_string(),
                "scanlation_group".to_string(),
            ],
            limit: 20,
            offset: (param.page.unwrap_or(1) as i64 - 1) * 20,
            title: param.keyword,
            order,
            ..Default::default()
        };

        let req = Request {
            method: "GET".to_string(),
            url: format!(
                "{}/manga?{}",
                self.url,
                serde_qs::to_string(&query).unwrap()
            ),
            headers: None,
        };

        let res = http_request(req);
        let res: Results = match serde_json::from_str(&res.body) {
            Ok(res) => res,
            Err(e) => {
                return ExtensionResult::err(format!("{}", e).as_str());
            }
        };

        let manga: Vec<Manga> = res
            .results
            .into_iter()
            .filter_map(Self::map_result_to_manga)
            .collect();

        ExtensionResult::ok(manga)
    }

    fn get_manga_info(&self, path: String) -> ExtensionResult<Manga> {
        let query = request::Manga {
            includes: vec![
                "cover_art".to_string(),
                "author".to_string(),
                "artist".to_string(),
            ],
        };
        let req = Request {
            method: "GET".to_string(),
            url: format!(
                "{}{}?{}",
                self.url,
                path,
                serde_qs::to_string(&query).unwrap()
            ),
            headers: None,
        };

        let res = http_request(req);
        let res = serde_json::from_str(&res.body);
        let manga: Manga = match res.map(Self::map_result_to_manga) {
            Ok(Some(res)) => res,
            Ok(_) | Err(_) => {
                return ExtensionResult::err("failed to parse manga");
            }
        };

        ExtensionResult::ok(manga)
    }

    fn get_chapters(&self, path: String) -> ExtensionResult<Vec<Chapter>> {
        let query = request::MangaFeed {
            limit: 500,
            translated_language: vec!["en".to_string()],
            ..Default::default()
        };
        let req = Request {
            method: "GET".to_string(),
            url: format!(
                "{}{}/feed?{}",
                self.url,
                path,
                serde_qs::to_string(&query).unwrap()
            ),
            headers: None,
        };

        let res = http_request(req);
        let res: Results = match serde_json::from_str(&res.body) {
            Ok(res) => res,
            Err(e) => {
                return ExtensionResult::err(format!("{}", e).as_str());
            }
        };

        let chapters: Vec<Chapter> = res
            .results
            .into_iter()
            .filter_map(Self::map_result_to_chapter)
            .collect();

        ExtensionResult::ok(chapters)
    }

    fn get_pages(&self, path: String) -> ExtensionResult<Vec<String>> {
        let req = Request {
            method: "GET".to_string(),
            url: format!("{}{}", self.url, path,),
            headers: None,
        };

        let res = http_request(req);
        let res = serde_json::from_str(&res.body);
        let pages = match res.map(Self::map_result_to_pages) {
            Ok(Some(pages)) => pages,
            Ok(_) | Err(_) => {
                return ExtensionResult::err("failed to parse pages");
            }
        };

        ExtensionResult::ok(pages)
    }
}
