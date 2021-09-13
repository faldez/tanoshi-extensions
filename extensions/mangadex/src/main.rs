use chrono::NaiveDateTime;
use data::Relationship;
use fancy_regex::Regex;
use tanoshi_lib::prelude::*;
use tanoshi_util::http::Request;

use crate::data::{
    manga::{request, ListOrder, Order},
    Home, Results, SingleResult,
};

mod data;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub static ID: i64 = 2;
pub static NAME: &str = "mangadex";
pub static URL: &str = "https://api.mangadex.org";

pub struct Mangadex;

register_extension!(Mangadex);

impl Default for Mangadex {
    fn default() -> Self {
        Self {}
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

    pub fn map_tags_to_string(relationships: Vec<Relationship>) -> Vec<String> {
        let mut tags = vec![];
        for relationship in relationships {
            if let data::Relationship::Tag { attributes, .. } = relationship {
                if let Some(name) = attributes.and_then(|attr| attr.name.get("en").cloned()) {
                    tags.push(name.to_owned());
                }
            };
        }

        tags
    }

    pub fn map_result_to_manga(data: Relationship) -> Option<Manga> {
        match data {
            data::Relationship::Manga {
                id,
                attributes,
                relationships,
            } => {
                let mut author = vec![];
                let mut genre = vec![];
                let mut file_name = "".to_string();
                for relationship in relationships {
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
                            if let Some(name) =
                                attributes.and_then(|attr| attr.name.get("en").cloned())
                            {
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

                Some(Manga {
                    source_id: ID,
                    title: attributes
                        .clone()
                        .and_then(|attr| {
                            if let Some(title) = attr.title.get("en").cloned() {
                                Some(title)
                            } else if let Some(title) = attr.title.get("ja").cloned() {
                                Some(title)
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| "".to_string()),
                    author,
                    genre: attributes
                        .clone()
                        .map(|attr| attr.tags)
                        .map(Self::map_tags_to_string)
                        .unwrap_or_else(|| vec![]),
                    status: attributes
                        .clone()
                        .and_then(|attr| attr.status)
                        .map(|s| s.to_string()),
                    description: attributes
                        .and_then(|attr| attr.description.get("en").cloned())
                        .map(|description| Self::remove_bbcode(&description)),
                    path: format!("/manga/{}", id),
                    cover_url: format!("https://uploads.mangadex.org/covers/{}/{}", id, file_name),
                })
            }
            _ => None,
        }
    }

    pub fn map_result_to_chapter(data: Relationship) -> Option<Chapter> {
        match data {
            data::Relationship::Chapter {
                id,
                attributes,
                relationships,
            } => {
                let mut scanlator = "".to_string();
                for relationship in relationships {
                    if let data::Relationship::ScanlationGroup { attributes, .. } = relationship {
                        if let Some(name) = attributes.map(|attr| attr.name) {
                            scanlator = name;
                        }
                    }
                }

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

    pub fn map_result_to_pages(data: Relationship) -> Option<Vec<String>> {
        match data {
            data::Relationship::Chapter { id, attributes, .. } => {
                if let Some(attr) = attributes {
                    let res =
                        Request::get(format!("{}/at-home/server/{}", URL, id,).as_str()).call();
                    if res.status > 299 {
                        return None;
                    }
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
            url: URL.to_string(),
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

        let res = Request::get(
            format!("{}/manga?{}", URL, serde_qs::to_string(&query).unwrap()).as_str(),
        )
        .call();
        if res.status > 299 {
            return ExtensionResult::err("http request error");
        }
        let res: Results = match serde_json::from_str(&res.body) {
            Ok(res) => res,
            Err(e) => {
                return ExtensionResult::err(format!("{}", e).as_str());
            }
        };

        let manga: Vec<Manga> = res
            .data
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
        let res = Request::get(
            format!("{}{}?{}", URL, path, serde_qs::to_string(&query).unwrap()).as_str(),
        )
        .call();
        if res.status > 299 {
            return ExtensionResult::err("http request error");
        }
        let res = serde_json::from_str::<SingleResult>(&res.body);
        let manga: Manga = match res.map(|res| Self::map_result_to_manga(res.data)) {
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
            includes: vec!["scanlation_group".to_string()],
            ..Default::default()
        };
        let res = Request::get(
            format!(
                "{}{}/feed?{}",
                URL,
                path,
                serde_qs::to_string(&query).unwrap()
            )
            .as_str(),
        )
        .call();
        if res.status > 299 {
            return ExtensionResult::err("http request error");
        }
        let res: Results = match serde_json::from_str(&res.body) {
            Ok(res) => res,
            Err(e) => {
                return ExtensionResult::err(format!("{}", e).as_str());
            }
        };

        let chapters: Vec<Chapter> = res
            .data
            .into_iter()
            .filter_map(Self::map_result_to_chapter)
            .collect();

        ExtensionResult::ok(chapters)
    }

    fn get_pages(&self, path: String) -> ExtensionResult<Vec<String>> {
        let res = Request::get(format!("{}{}", URL, path,).as_str()).call();
        if res.status > 299 {
            return ExtensionResult::err("http request error");
        }
        let res = serde_json::from_str::<SingleResult>(&res.body);
        let pages = match res.map(|res| Self::map_result_to_pages(res.data)) {
            Ok(Some(pages)) => pages,
            Ok(_) | Err(_) => {
                return ExtensionResult::err("failed to parse pages");
            }
        };

        ExtensionResult::ok(pages)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_manga_list() {
        let mangadex = Mangadex::default();

        let res = mangadex.get_manga_list(Param::default());

        assert_eq!(res.data.is_some(), true);
        assert_eq!(res.error.is_none(), true);
    }

    #[test]
    fn test_get_manga_list_latest() {
        let mangadex = Mangadex::default();

        let res = mangadex.get_manga_list(Param {
            sort_by: Some(SortByParam::LastUpdated),
            sort_order: Some(SortOrderParam::Desc),
            ..Default::default()
        });

        assert!(res.error.is_none(), "should not error but {:?}", res.error);
        assert!(res.data.is_some());
    }

    #[test]
    fn test_get_manga() {
        let mangadex = Mangadex::default();

        let res =
            mangadex.get_manga_info("/manga/77bee52c-d2d6-44ad-a33a-1734c1fe696a".to_string());

        assert_eq!(res.error, None, "should be None, but got {:?}", res.error);
        assert!(res.data.is_some());

        if let Some(manga) = res.data {
            assert_eq!(manga.title, "Kage no Jitsuryokusha ni Naritakute");
            assert_eq!(manga.cover_url, "https://uploads.mangadex.org/covers/77bee52c-d2d6-44ad-a33a-1734c1fe696a/2273a826-f8a6-45f4-ae68-7acc714934d1.jpg");
            assert_eq!(
                manga.genre,
                vec![
                    "Reincarnation",
                    "Action",
                    "Demons",
                    "Comedy",
                    "Martial Arts",
                    "Magic",
                    "Harem",
                    "Isekai",
                    "Drama",
                    "School Life",
                    "Fantasy",
                    "Adaptation"
                ]
            );
        }
    }

    #[test]
    fn test_get_chapters() {
        let mangadex = Mangadex::default();

        let res = mangadex.get_chapters("/manga/77bee52c-d2d6-44ad-a33a-1734c1fe696a".to_string());

        assert_eq!(res.error, None, "should be None, but got {:?}", res.error);
        assert!(res.data.is_some());
    }

    #[test]
    fn test_get_pages() {
        let mangadex = Mangadex::default();

        let res = mangadex.get_pages("/chapter/11a3c024-dcba-4f0c-9ea1-4361302775de".to_string());

        assert_eq!(res.error, None, "should be None, but got {:?}", res.error);
        assert!(res.data.is_some());
    }
}
