use std::str::FromStr;

use chrono::{NaiveDate, NaiveDateTime};
use tanoshi_lib::prelude::*;
use tanoshi_util::http::Request;

const ID: i64 = 8;
const NAME: &str = "manhwa18_cc";
const URL: &str = "https://manhwa18.cc";

#[derive(Debug, Default)]
pub struct Manhwa18;

register_extension!(Manhwa18);

macro_rules! create_selector {
    ($arg:literal) => {
        match scraper::Selector::parse($arg).map_err(|e| format!("{:?}", e)) {
            Ok(selector) => selector,
            Err(e) => {
                return ExtensionResult::err(&e);
            }
        }
    };
}

impl Extension for Manhwa18 {
    fn detail(&self) -> Source {
        Source {
            id: ID,
            name: NAME.to_string(),
            url: URL.to_string(),
            version: Version::from_str(env!("CARGO_PKG_VERSION")).unwrap_or_default(),
            lib_version: tanoshi_lib::VERSION.to_owned(),
            icon: "https://manhwa18.cc/images/favicon-160x160.png".to_string(),
            need_login: false,
            languages: vec!["en".to_string()],
        }
    }

    fn filters(&self) -> ExtensionResult<Option<Filters>> {
        ExtensionResult::ok(None)
    }

    fn get_manga_list(&self, param: Param) -> ExtensionResult<Vec<Manga>> {
        let sort_by = param.sort_by.unwrap_or(SortByParam::Views);

        let query = match sort_by {
            SortByParam::LastUpdated => "latest".to_string(),
            SortByParam::Title => "alphabet".to_string(),
            _ => "trending".to_string(),
        };

        let url = if let Some(keyword) = param.keyword {
            let page = param.page.unwrap_or(1);
            if page > 2 {
                format!("{}/search/{}?q={}", URL, page, keyword)
            } else {
                format!("{}/search?q={}", URL, keyword)
            }
        } else {
            format!(
                "{}/webtoons/{}?orderby={}",
                URL,
                param.page.unwrap_or(1),
                query
            )
        };

        let resp = Request::get(&url).call();
        if resp.status > 299 {
            return ExtensionResult::err(format!("http request to {} error", url).as_str());
        }

        let document = scraper::Html::parse_document(&resp.body);

        let mut manga = vec![];
        let selector = create_selector!(".manga-item a[href^=\"/webtoon\"] img");
        for element in document.select(&selector) {
            let cover_url = element
                .value()
                .attr("data-src")
                .map(|s| s.to_string())
                .unwrap_or_else(|| "".to_string());

            let parent = element.parent().unwrap().value().as_element().unwrap();

            let title = parent
                .attr("title")
                .map(|s| s.to_string())
                .unwrap_or_else(|| "".to_string());

            let path = parent
                .attr("href")
                .map(|s| s.to_string())
                .unwrap_or_else(|| "".to_string());

            manga.push(Manga {
                source_id: ID,
                title,
                author: vec![],
                genre: vec![],
                status: None,
                description: None,
                path,
                cover_url,
            })
        }

        ExtensionResult::ok(manga)
    }

    /// Get the rest of details unreachable from `get_mangas`
    fn get_manga_info(&self, path: String) -> ExtensionResult<Manga> {
        let resp = Request::get(format!("{}{}", URL, &path).as_str()).call();
        if resp.status > 299 {
            return ExtensionResult::err("http request error");
        }
        let document = scraper::Html::parse_document(&resp.body);

        let selector = create_selector!("a[href^=\"/webtoon\"] img");
        let title = document
            .select(&selector)
            .next()
            .and_then(|el| el.value().attr("title"))
            .map(|title| title.to_string())
            .unwrap_or_else(|| "".to_string());

        let cover_url = document
            .select(&selector)
            .next()
            .and_then(|el| el.value().attr("data-src"))
            .map(|src| src.to_string())
            .unwrap_or_else(|| "".to_string());

        let mut authors = vec![];
        let selector = create_selector!(".artist-content a");
        for el in document.select(&selector) {
            if let Some(author) = el.text().next() {
                authors.push(author.to_string());
            }
        }

        let selector = create_selector!(".artist-content a");
        for el in document.select(&selector) {
            if let Some(artist) = el.text().next() {
                authors.push(artist.to_string());
            }
        }

        let selector = create_selector!(".dsct > p");
        let description = document
            .select(&selector)
            .next()
            .and_then(|el| el.text().next())
            .map(|src| src.to_string());

        let mut genres = vec![];
        let selector = create_selector!(".genres-content a");
        for el in document.select(&selector) {
            if let Some(genre) = el.text().next() {
                genres.push(genre.to_string());
            }
        }

        let manga = Manga {
            source_id: ID,
            title,
            author: authors,
            genre: genres,
            status: Some("Ongoing".to_string()),
            description,
            path,
            cover_url,
        };

        ExtensionResult::ok(manga)
    }

    fn get_chapters(&self, path: String) -> ExtensionResult<Vec<Chapter>> {
        let resp = Request::get(format!("{}{}", URL, &path).as_str()).call();
        if resp.status > 299 {
            return ExtensionResult::err("http request error");
        }

        let document = scraper::Html::parse_document(&resp.body);

        let mut chapters = vec![];

        let selector = create_selector!("#chapterlist .a-h.wleft");
        for el in document.select(&selector) {
            let selector = create_selector!(".chapter-name");
            let title = el
                .select(&selector)
                .next()
                .and_then(|el| el.text().next())
                .map(|txt| txt.to_string())
                .unwrap_or_else(|| "".to_string());
            let path = el
                .select(&selector)
                .next()
                .and_then(|el| el.value().attr("href"))
                .map(|txt| txt.to_string())
                .unwrap_or_else(|| "".to_string());
            let selector = create_selector!(".chapter-time");
            let uploaded = el
                .select(&selector)
                .next()
                .and_then(|el| el.text().next())
                .and_then(|txt| NaiveDate::parse_from_str(txt, "%d %b %Y").ok())
                .map(|date| date.and_hms(0, 0, 0))
                .unwrap_or_else(|| NaiveDateTime::from_timestamp(0, 0));

            chapters.push(Chapter {
                source_id: ID,
                title: title.clone(),
                path,
                number: title.replace("Chapter ", "").parse().unwrap_or_default(),
                scanlator: "".to_string(),
                uploaded,
            })
        }

        ExtensionResult::ok(chapters)
    }

    fn get_pages(&self, path: String) -> ExtensionResult<Vec<String>> {
        let resp = Request::get(format!("{}{}", URL, path).as_str()).call();
        if resp.status > 299 {
            return ExtensionResult::err("http request error");
        }
        let document = scraper::Html::parse_document(&resp.body);

        let selector = create_selector!(".read-content > img");

        let pages = document
            .select(&selector)
            .into_iter()
            .map(|el| {
                el.value()
                    .attr("src")
                    .map(|src| src.to_string())
                    .unwrap_or_else(|| "".to_string())
            })
            .collect();

        ExtensionResult::ok(pages)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_manga_list() {
        let ext = Manhwa18::default();

        let res = ext.get_manga_list(Param::default());

        assert_eq!(res.error, None, "should be none, got {:?}", res.error);
        assert!(res.data.is_some());

        if let Some(items) = res.data {
            for item in items {
                assert_eq!(item.title.is_empty(), false);
                assert_eq!(item.path.is_empty(), false);
                assert_eq!(
                    item.cover_url.is_empty(),
                    false,
                    "on manga {} got {}",
                    item.title,
                    item.cover_url
                );
            }
        }
    }

    #[test]
    fn test_search_manga_list() {
        let ext = Manhwa18::default();

        let res = ext.get_manga_list(Param {
            keyword: Some("unwanted".to_string()),
            ..Default::default()
        });

        assert_eq!(res.error, None, "should be none, got {:?}", res.error);
        assert!(res.data.is_some());

        if let Some(items) = res.data {
            for item in items {
                assert_eq!(item.title, "The Unwanted Roommate");
                assert_eq!(item.path.is_empty(), false);
                assert_eq!(
                    item.cover_url.is_empty(),
                    false,
                    "on manga {} got {}",
                    item.title,
                    item.cover_url
                );
            }
        }
    }

    #[test]
    fn test_get_manga() {
        let ext = Manhwa18::default();

        let res = ext.get_manga_info("/webtoon/secret-class".to_string());

        assert_eq!(res.error, None, "should be none, got {:?}", res.error);
        assert!(res.data.is_some());

        let data = res.data.unwrap();
        assert_eq!(
            data.title, "Secret Class",
            "should be Secret Class, got {:?}",
            data.title
        );

        let cover_url = data.cover_url.clone();
        assert_eq!(
            cover_url.clone(),
            "https://manhwa18.cc/manga/secret-classczv.jpg".to_string(),
            "should be https://manhwa18.cc/manga/secret-classczv.jpg, got {:?}",
            cover_url.clone()
        );
    }

    #[test]
    fn test_get_chapters() {
        let ext = Manhwa18::default();

        let res = ext.get_chapters("/webtoon/secret-class".to_string());
        assert_eq!(res.error, None, "should be none, got {:?}", res.error);
        assert!(res.data.is_some());
    }

    #[test]
    fn test_get_pages() {
        let ext = Manhwa18::default();

        let res = ext.get_pages("/webtoon/secret-class/chapter-103".to_string());
        assert_eq!(res.error, None, "should be none, got {:?}", res.error);
        assert!(res.data.is_some());
    }
}
