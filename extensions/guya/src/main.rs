mod data;

use std::{collections::HashMap, str::FromStr};

use chrono::NaiveDateTime;
use tanoshi_lib::prelude::*;
use tanoshi_util::http::Request;

use crate::data::{Detail, Series};

const ID: i64 = 7;
const NAME: &str = "guya";
const URL: &str = "https://guya.moe";

#[derive(Debug, Default)]
pub struct Guya;

register_extension!(Guya);

impl Extension for Guya {
    fn detail(&self) -> Source {
        Source {
            id: ID,
            name: NAME.to_string(),
            url: URL.to_string(),
            version: Version::from_str(env!("CARGO_PKG_VERSION")).unwrap_or_default(),
            lib_version: tanoshi_lib::VERSION.to_owned(),
            icon: "https://guya.moe/static/logo_small.png".to_string(),
            need_login: false,
            languages: vec!["en".to_string()],
        }
    }

    fn filters(&self) -> ExtensionResult<Option<Filters>> {
        ExtensionResult::ok(None)
    }

    fn get_manga_list(&self, _param: Param) -> ExtensionResult<Vec<Manga>> {
        let resp = Request::get(format!("{}/api/get_all_series", URL).as_str()).call();
        if resp.status > 299 {
            return ExtensionResult::err("http request error");
        }

        let results: HashMap<String, Detail> = match serde_json::from_str(&resp.body) {
            Ok(res) => res,
            Err(e) => {
                return ExtensionResult::err(format!("{}", e).as_str());
            }
        };

        let manga: Vec<Manga> = results
            .into_iter()
            .map(|(title, detail)| Manga {
                source_id: ID,
                title,
                author: vec![detail.author, detail.artist],
                genre: vec![],
                status: Some("Ongoing".to_string()),
                description: Some(detail.description),
                path: format!("/api/series/{}", detail.slug),
                cover_url: format!("{}{}", URL, detail.cover),
            })
            .collect();

        ExtensionResult::ok(manga)
    }

    /// Get the rest of details unreachable from `get_mangas`
    fn get_manga_info(&self, path: String) -> ExtensionResult<Manga> {
        let resp = Request::get(format!("{}{}", URL, &path).as_str()).call();
        if resp.status > 299 {
            return ExtensionResult::err("http request error");
        }

        let series: Series = match serde_json::from_str(&resp.body) {
            Ok(detail) => detail,
            Err(e) => {
                return ExtensionResult::err(format!("{}", e).as_str());
            }
        };

        let manga = Manga {
            source_id: ID,
            title: series.title.clone(),
            author: vec![series.author.clone(), series.author.clone()],
            genre: vec![],
            status: Some("Ongoing".to_string()),
            description: Some(series.description.clone()),
            path,
            cover_url: format!("{}{}", URL, series.cover),
        };

        ExtensionResult::ok(manga)
    }

    fn get_chapters(&self, path: String) -> ExtensionResult<Vec<Chapter>> {
        let resp = Request::get(format!("{}{}", URL, &path).as_str()).call();
        if resp.status > 299 {
            return ExtensionResult::err("http request error");
        }

        let series: Series = match serde_json::from_str(&resp.body) {
            Ok(detail) => detail,
            Err(e) => {
                return ExtensionResult::err(format!("{}", e).as_str());
            }
        };

        let mut chapters = vec![];

        for (number, chapter) in series.chapters {
            chapters.push(Chapter {
                source_id: ID,
                title: chapter.title.clone(),
                path: format!("{}/{}", path, number),
                number: number.parse().unwrap_or_default(),
                scanlator: if let Some(group) = chapter.groups.into_keys().next() {
                    series
                        .groups
                        .clone()
                        .get(&group)
                        .cloned()
                        .unwrap_or_else(|| "".to_string())
                } else {
                    "".to_string()
                },
                uploaded: if let Some(date) = chapter.release_date.into_values().next() {
                    NaiveDateTime::from_timestamp(date as i64, 0)
                } else {
                    NaiveDateTime::from_timestamp(0, 0)
                },
            })
        }

        ExtensionResult::ok(chapters)
    }

    fn get_pages(&self, path: String) -> ExtensionResult<Vec<String>> {
        //https://guya.moe/media/manga/Kaguya-Wants-To-Be-Confessed-To/chapters/0236_5ts2bsnf/7/02.png?v2
        let split: Vec<_> = path.rsplitn(2, '/').collect();
        let resp = Request::get(format!("{}{}", URL, split[1]).as_str()).call();
        if resp.status > 299 {
            return ExtensionResult::err("http request error");
        }

        let series: Series = match serde_json::from_str(&resp.body) {
            Ok(detail) => detail,
            Err(e) => {
                return ExtensionResult::err(format!("{}", e).as_str());
            }
        };

        let pages = series
            .chapters
            .get(split[0])
            .and_then(|chapter| {
                chapter
                    .groups
                    .iter()
                    .next()
                    .map(|(group, pages)| (chapter.folder.clone(), group, pages))
            })
            .map(|(folder, group, pages)| {
                pages
                    .iter()
                    .map(|page| {
                        format!(
                            "{}/media/manga/Kaguya-Wants-To-Be-Confessed-To/chapters/{}/{}/{}",
                            URL, folder, group, page
                        )
                    })
                    .collect()
            })
            .unwrap_or(vec![]);

        ExtensionResult::ok(pages)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_manga_list() {
        let ext = Guya::default();

        let res = ext.get_manga_list(Param::default());

        assert_eq!(res.error, None, "should be none, got {:?}", res.error);
        assert!(res.data.is_some());
    }

    #[test]
    fn test_get_manga() {
        let ext = Guya::default();

        let res = ext.get_manga_info("/api/series/Kaguya-Wants-To-Be-Confessed-To/".to_string());

        assert_eq!(res.error, None, "should be none, got {:?}", res.error);
        assert!(res.data.is_some());
    }

    #[test]
    fn test_get_chapters() {
        let ext = Guya::default();

        let res = ext.get_chapters("/api/series/Kaguya-Wants-To-Be-Confessed-To/".to_string());
        assert_eq!(res.error, None, "should be none, got {:?}", res.error);
        assert!(res.data.is_some());
    }

    #[test]
    fn test_get_pages() {
        let ext = Guya::default();

        let res = ext.get_pages("/api/series/Kaguya-Wants-To-Be-Confessed-To/1".to_string());
        assert_eq!(res.error, None, "should be none, got {:?}", res.error);
        assert!(res.data.is_some());
    }
}
