use chrono::NaiveDateTime;
use fancy_regex::Regex;
use scraper::{Html, Selector};
use tanoshi_lib::prelude::*;
use tanoshi_util::http::Request;

pub static ID: i64 = 6;
pub static NAME: &str = "nhentai";
pub static URL: &str = "https://nhentai.net";

#[derive(Debug, Default)]
struct Nhentai {}

register_extension!(Nhentai);

impl Extension for Nhentai {
    fn detail(&self) -> Source {
        Source {
            id: ID,
            name: NAME.to_string(),
            url: URL.to_string(),
            version: std::env!("PLUGIN_VERSION").to_string(),
            icon: "https://static.nhentai.net/img/logo.090da3be7b51.svg".to_string(),
            need_login: false,
            languages: vec!["en".to_string()],
        }
    }

    fn filters(&self) -> ExtensionResult<Option<Filters>> {
        ExtensionResult::ok(None)
    }

    fn get_manga_list(&self, param: Param) -> ExtensionResult<Vec<Manga>> {
        let sort_by = param
            .sort_by
            .map(|sort_by| match sort_by {
                SortByParam::Views => "popular",
                _ => "",
            })
            .unwrap_or("popular");
        let page = param.page.unwrap_or(1);

        let url = if let Some(keyword) = param
            .keyword
            .and_then(|keyword| (!keyword.is_empty()).then(|| keyword))
        {
            let sort = if !sort_by.is_empty() {
                format!("&sort={}", sort_by)
            } else {
                "".to_string()
            };
            format!("{}/search?q={}{}&page={}", URL, keyword, sort, page)
        } else {
            format!("{}/language/english/{}?page={}", URL, sort_by, page)
        };

        let res = Request::get(&url).call();
        if res.status > 299 {
            return ExtensionResult::err("http request error");
        }

        let document = Html::parse_document(&res.body);
        let gallery_selector = match Selector::parse(".gallery") {
            Ok(selector) => selector,
            Err(e) => {
                return ExtensionResult::err(format!("error parse selector: {:?}", e).as_str())
            }
        };
        let thumbnail_selector = match Selector::parse("a > img") {
            Ok(selector) => selector,
            Err(e) => {
                return ExtensionResult::err(format!("error parse selector: {:?}", e).as_str())
            }
        };
        let path_selector = match Selector::parse("a") {
            Ok(selector) => selector,
            Err(e) => {
                return ExtensionResult::err(format!("error parse selector: {:?}", e).as_str())
            }
        };
        let title_selector = match Selector::parse("a > .caption") {
            Ok(selector) => selector,
            Err(e) => {
                return ExtensionResult::err(format!("error parse selector: {:?}", e).as_str())
            }
        };

        let mut manga_list = vec![];
        for gallery in document.select(&gallery_selector) {
            let mut manga = Manga {
                source_id: ID,
                status: Some("ongoing".to_string()),
                ..Default::default()
            };
            if let Some(thumbnail) = gallery.select(&thumbnail_selector).next() {
                if let Some(cover_url) = thumbnail.value().attr("data-src") {
                    manga.cover_url = cover_url.to_string();
                }
            }
            if let Some(link) = gallery.select(&path_selector).next() {
                if let Some(href) = link.value().attr("href") {
                    manga.path = href.to_string();
                }
            }
            if let Some(caption) = gallery.select(&title_selector).next() {
                if let Some(title) = caption.text().next() {
                    manga.title = title.to_string();
                }
            }

            manga_list.push(manga);
        }

        ExtensionResult::ok(manga_list)
    }

    fn get_manga_info(&self, path: String) -> ExtensionResult<Manga> {
        let res = Request::get(format!("{}{}", URL, path).as_str()).call();
        if res.status > 299 {
            return ExtensionResult::err("http request error");
        }

        let document = Html::parse_document(&res.body);
        let gallery_id_selector = match Selector::parse("h3[id=\"gallery_id\"]") {
            Ok(selector) => selector,
            Err(e) => {
                return ExtensionResult::err(format!("error parse selector: {:?}", e).as_str())
            }
        };
        let parodies_selector = match Selector::parse("a[href^=\"/parody/\"] > .name") {
            Ok(selector) => selector,
            Err(e) => {
                return ExtensionResult::err(format!("error parse selector: {:?}", e).as_str())
            }
        };
        let characters_selector = match Selector::parse("a[href^=\"/character/\"] > .name") {
            Ok(selector) => selector,
            Err(e) => {
                return ExtensionResult::err(format!("error parse selector: {:?}", e).as_str())
            }
        };
        let languages_selector = match Selector::parse("a[href^=\"/language/\"] > .name") {
            Ok(selector) => selector,
            Err(e) => {
                return ExtensionResult::err(format!("error parse selector: {:?}", e).as_str())
            }
        };
        let categories_selector = match Selector::parse("a[href^=\"/category/\"] > .name") {
            Ok(selector) => selector,
            Err(e) => {
                return ExtensionResult::err(format!("error parse selector: {:?}", e).as_str())
            }
        };
        let pages_selector = match Selector::parse("a[href^=\"/search/?q=pages\"] > .name") {
            Ok(selector) => selector,
            Err(e) => {
                return ExtensionResult::err(format!("error parse selector: {:?}", e).as_str())
            }
        };
        let thumbnail_selector = match Selector::parse("#cover > a > img") {
            Ok(selector) => selector,
            Err(e) => {
                return ExtensionResult::err(format!("error parse selector: {:?}", e).as_str())
            }
        };
        let title_selector = match Selector::parse("h1.title > .pretty") {
            Ok(selector) => selector,
            Err(e) => {
                return ExtensionResult::err(format!("error parse selector: {:?}", e).as_str())
            }
        };
        let author_selector = match Selector::parse("a[href^=\"/artist/\"] > .name") {
            Ok(selector) => selector,
            Err(e) => {
                return ExtensionResult::err(format!("error parse selector: {:?}", e).as_str())
            }
        };
        let genre_selector = match Selector::parse("a[href^=\"/tag/\"] > .name") {
            Ok(selector) => selector,
            Err(e) => {
                return ExtensionResult::err(format!("error parse selector: {:?}", e).as_str())
            }
        };

        let mut manga = Manga {
            source_id: ID,
            status: Some("ongoing".to_string()),
            path,
            description: None,
            ..Default::default()
        };

        let mut description = "".to_string();
        if let Some(gallery_id) = document.select(&gallery_id_selector).next().map(|el| {
            el.text()
                .into_iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
                .join("")
        }) {
            description = format!("{}", gallery_id);
        }
        let parodies = document
            .select(&parodies_selector)
            .into_iter()
            .filter_map(|el| el.text().next())
            .collect::<Vec<&str>>()
            .join(",");
        if !parodies.is_empty() {
            description = format!("{}\nParodies: {}", description, parodies);
        }
        let characters = document
            .select(&characters_selector)
            .into_iter()
            .filter_map(|el| el.text().next())
            .collect::<Vec<&str>>()
            .join(",");
        if !characters.is_empty() {
            description = format!("{}\nCharacters: {}", description, characters);
        }
        let languages = document
            .select(&languages_selector)
            .into_iter()
            .filter_map(|el| el.text().next())
            .collect::<Vec<&str>>()
            .join(",");
        if !languages.is_empty() {
            description = format!("{}\nLanguages: {}", description, languages);
        }
        let categories = document
            .select(&categories_selector)
            .into_iter()
            .filter_map(|el| el.text().next())
            .collect::<Vec<&str>>()
            .join(",");
        if !categories.is_empty() {
            description = format!("{}\nCategories: {}", description, categories);
        }
        if let Some(pages) = document.select(&pages_selector).next().map(|el| {
            el.text()
                .into_iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
                .join("")
        }) {
            description = format!("{}\nPages: {}", description, pages);
        }

        manga.description = Some(description);

        if let Some(thumbnail) = document.select(&thumbnail_selector).next() {
            if let Some(cover_url) = thumbnail.value().attr("data-src") {
                manga.cover_url = cover_url.to_string();
            }
        }
        if let Some(h1) = document.select(&title_selector).next() {
            if let Some(title) = h1.text().next() {
                manga.title = title.to_string();
            }
        }
        for author in document.select(&author_selector) {
            if let Some(name) = author.text().next() {
                manga.author.push(name.to_string());
            }
        }
        for tag in document.select(&genre_selector) {
            if let Some(name) = tag.text().next() {
                manga.genre.push(name.to_string());
            }
        }

        ExtensionResult::ok(manga)
    }

    fn get_chapters(&self, path: String) -> ExtensionResult<Vec<Chapter>> {
        let res = Request::get(format!("{}{}", URL, path).as_str()).call();
        if res.status > 299 {
            return ExtensionResult::err("http request error");
        }

        let document = Html::parse_document(&res.body);
        let scanlator_selector = match Selector::parse("a[href^=\"/group/\"] > .name") {
            Ok(selector) => selector,
            Err(e) => {
                return ExtensionResult::err(format!("error parse selector: {:?}", e).as_str())
            }
        };
        let uploaded_selector = match Selector::parse(".tags > time") {
            Ok(selector) => selector,
            Err(e) => {
                return ExtensionResult::err(format!("error parse selector: {:?}", e).as_str())
            }
        };
        let group = if let Some(group) = document.select(&scanlator_selector).next() {
            group.text().next().map(|group| group.to_string())
        } else {
            None
        };
        let uploaded = if let Some(uploaded) = document.select(&uploaded_selector).next() {
            uploaded
                .value()
                .attr("datetime")
                .and_then(|t| NaiveDateTime::parse_from_str(t, "%Y-%m-%dT%H:%M:%S%.f%z").ok())
        } else {
            None
        };

        let chapter = Chapter {
            source_id: ID,
            title: "Chapter 1".to_string(),
            path,
            number: 1_f64,
            scanlator: group.unwrap_or_else(|| "".to_string()),
            uploaded: uploaded.unwrap_or_else(|| NaiveDateTime::from_timestamp(0, 0)),
        };

        ExtensionResult::ok(vec![chapter])
    }

    fn get_pages(&self, path: String) -> ExtensionResult<Vec<String>> {
        let res = Request::get(format!("{}{}", URL, path).as_str()).call();
        if res.status > 299 {
            return ExtensionResult::err("http request error");
        }

        let document = Html::parse_document(&res.body);
        let page_selector = match Selector::parse(".thumb-container > .gallerythumb > img") {
            Ok(selector) => selector,
            Err(e) => {
                return ExtensionResult::err(format!("error parse selector: {:?}", e).as_str())
            }
        };

        let mut pages = vec![];
        for thumb in document.select(&page_selector) {
            if let Some(url) = thumb.value().attr("data-src") {
                let page = Regex::new(r#"(\d+)\/(\d+)t.(.+)$"#)
                    .unwrap()
                    .replace_all(url, "${1}/${2}.${3}")
                    .to_string();
                pages.push(page.replace("t.nhentai", "i.nhentai"));
            }
        }

        ExtensionResult::ok(pages)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_manga_info() {
        let nhentai = Nhentai::default();

        let res = nhentai.get_manga_info("/g/379261/".to_string());

        assert_eq!(res.error, None, "should None get {:?}", res.error);

        let data = res.data.unwrap();

        assert_eq!(data.description, Some("#379261\nParodies: girls frontline\nCharacters: rpk-16\nLanguages: translated,english\nCategories: doujinshi\nPages: 15".to_string()));
    }

    #[test]
    fn test_get_chapters() {
        let nhentai = Nhentai::default();

        let res = nhentai.get_chapters("/g/370978/".to_string());

        assert_eq!(res.error, None, "should None get {:?}", res.error);

        if let Some(data) = res.data {
            if let Some(data) = data.get(0) {
                assert_eq!(
                    Ok(data.uploaded),
                    NaiveDateTime::parse_from_str(
                        "2021-08-28T21:50:59.973874+00:00",
                        "%Y-%m-%dT%H:%M:%S%.f%z"
                    )
                );
            }
        }
    }
}
