mod util;

use crate::util::*;

use fancy_regex::Regex;
use tanoshi_lib::prelude::*;
use tanoshi_util::http::Request;

pub static ID: i64 = 4;
pub static NAME: &str = "mangalife";

pub struct Mangalife {
    url: String,
}

register_extension!(Mangalife);

impl Default for Mangalife {
    fn default() -> Self {
        Self {
            url: "https://manga4life.com".to_string(),
        }
    }
}

impl Mangalife {
    fn find_filter_map_value(
        filter_map: &Option<Filters>,
        key: &str,
        index: usize,
    ) -> Result<FilterValue, Box<dyn std::error::Error>> {
        Ok(filter_map
            .clone()
            .ok_or("no filters")?
            .fields
            .get(key)
            .ok_or(format!("no {} filter", key))?
            .values
            .clone()
            .ok_or("no possible values")?
            .get(index)
            .ok_or(format!("no value at index {}", index))?
            .clone())
    }
}

macro_rules! create_selector {
    ($arg:literal) => {
        match scraper::Selector::parse($arg).map_err(|e| format!("{:?}", e)) {
            Ok(selector) => selector,
            Err(e) => {
                return ExtensionResult::err(&e);
            }
        };
    };
}

macro_rules! regex_find {
    ($arg:literal, $text:ident) => {
        match Regex::new($arg).map(|re| re.find(&$text)) {
            Ok(Ok(Some(mat))) => mat.as_str().to_string(),
            Ok(Ok(None)) => {
                return ExtensionResult::err("regex not found anything");
            }
            Ok(Err(e)) | Err(e) => {
                return ExtensionResult::err(format!("error regex: {}", e).as_str());
            }
        };
    };
}

impl Extension for Mangalife {
    fn detail(&self) -> Source {
        Source {
            id: ID,
            name: NAME.to_string(),
            url: self.url.clone(),
            version: std::env!("PLUGIN_VERSION").to_string(),
            icon: "https://manga4life.com/media/favicon.png".to_string(),
            need_login: false,
            languages: vec!["en".to_string()],
        }
    }

    fn filters(&self) -> ExtensionResult<Option<Filters>> {
        ExtensionResult::ok(None)
    }

    fn get_manga_list(&self, param: Param) -> ExtensionResult<Vec<Manga>> {
        let vm_dir = {
            let resp = Request::get(format!("{}/search", &self.url).as_str()).call();
            if resp.status > 299 {
                return ExtensionResult::err("http request error");
            }
            let html = resp.body;
            if let Some(i) = html.find("vm.Directory =") {
                let dir = &html[i + 15..];
                if let Some(i) = dir.find("}];") {
                    let vm_dir = &dir[..i + 2];
                    vm_dir.to_string()
                } else {
                    return ExtensionResult::err("error get manga");
                }
            } else {
                return ExtensionResult::err("list not found");
            }
        };

        let mut dirs = match serde_json::from_str::<Vec<Dir>>(&vm_dir) {
            Ok(dirs) => dirs,
            Err(e) => {
                return ExtensionResult::err(format!("error parse json: {}", e).as_str());
            }
        };

        // let filter_map: Option<Filters> = ron::from_str(include_str!("filters.ron")).ok();
        // if let Some(filters) = param.filters {
        //     for (key, values) in filters {
        //         match key.as_str() {
        //             "name" => {
        //                 if let Some(name) = values.first() {
        //                     dirs.retain(|d| d.s.to_lowercase().contains(&name.to_lowercase()))
        //                 }
        //             }
        //             "author" => dirs.retain(move |d| {
        //                 let mut found = true;
        //                 for name in values.clone() {
        //                     for a in d.a.clone() {
        //                         if name != a {
        //                             found = false;
        //                             break;
        //                         }
        //                     }
        //                 }
        //                 found
        //             }),
        //             "year" => {
        //                 if let Some(name) = values.first() {
        //                     dirs.retain(|d| d.y.to_lowercase().contains(&name.to_lowercase()))
        //                 }
        //             }
        //             "status" => {
        //                 if let Some(name) = values.first() {
        //                     dirs.retain(|d| d.ss.to_lowercase().contains(&name.to_lowercase()))
        //                 }
        //             }
        //             "pstatus" => {
        //                 if let Some(name) = values.first() {
        //                     dirs.retain(|d| d.ps.to_lowercase().contains(&name.to_lowercase()))
        //                 }
        //             }
        //             "sort" => {
        //                 if let Some(value) = values.first() {
        //                     let index = match value.parse::<usize>() {
        //                         Ok(val) => val,
        //                         Err(e) => {
        //                             return ExtensionResult::err(
        //                                 format!("error parse value: {}", e).as_str(),
        //                             );
        //                         }
        //                     };

        //                     let value = match Self::find_filter_map_value(&filter_map, &key, index) {
        //                         Ok(value) => value,
        //                         Err(e) => {
        //                             return ExtensionResult::err(format!("error: {}", e).as_str());
        //                         }
        //                     };

        //                     dirs.sort_by_key(|d| d.field_by_name(&value.clone().value.unwrap_or("".to_string())));
        //                     if let Some(related) = value.related {
        //                         if let Some(desc) = related.get("desc") {
        //                             if desc == "true" {
        //                                 dirs.reverse();
        //                             }
        //                         }
        //                     }
        //                 }
        //             }
        //             "official" => {
        //                 if let Some(name) = values.first() {
        //                     dirs.retain(|d| d.o.to_lowercase().contains(&name.to_lowercase()))
        //                 }
        //             }
        //             "type" => {
        //                 if let Some(name) = values.first() {
        //                     dirs.retain(|d| d.t.to_lowercase().contains(&name.to_lowercase()))
        //                 }
        //             }
        //             "genre" => dirs.retain(move |d| {
        //                 let mut found = false;
        //                 for name in values.clone() {
        //                     for a in d.a.clone() {
        //                         if name == a {
        //                             found = true;
        //                             break;
        //                         }
        //                     }
        //                 }
        //                 found
        //             }),
        //             _ => {}
        //         }
        //     }
        // }

        let sort_by = param.sort_by.unwrap_or(SortByParam::Views);
        let sort_order = param.sort_order.unwrap_or(SortOrderParam::Asc);

        if let Some(keyword) = param.keyword {
            dirs.retain(|d| d.s.to_lowercase().contains(&keyword.to_lowercase()))
        }

        match sort_by {
            SortByParam::Views => {
                dirs.sort_by(|a, b| {
                    let v_a = a.v.parse::<i32>().unwrap_or_default();
                    let v_b = b.v.parse::<i32>().unwrap_or_default();
                    match sort_order {
                        SortOrderParam::Asc => v_a.cmp(&v_b),
                        SortOrderParam::Desc => v_b.cmp(&v_a),
                    }
                });
            }
            SortByParam::Comment => {}
            SortByParam::LastUpdated => {
                dirs.sort_by(|a, b| match sort_order {
                    SortOrderParam::Asc => a.lt.cmp(&b.lt),
                    SortOrderParam::Desc => b.lt.cmp(&a.lt),
                });
            }
            SortByParam::Title => {
                dirs.sort_by(|a, b| match sort_order {
                    SortOrderParam::Asc => a.s.cmp(&b.s),
                    SortOrderParam::Desc => b.s.cmp(&a.s),
                });
            }
        }

        let page = param.page.map(|p| p as usize).unwrap_or(1);
        let offset = (page - 1) * 20;
        if offset >= dirs.len() {
            return ExtensionResult::err("no page");
        }
        let mangas = match dirs[offset..].len() {
            0..=20 => &dirs[offset..],
            _ => &dirs[offset..offset + 20],
        };

        return ExtensionResult::ok(mangas.iter().map(|d| d.into()).collect());
    }

    /// Get the rest of details unreachable from `get_mangas`
    fn get_manga_info(&self, path: String) -> ExtensionResult<Manga> {
        let resp = Request::get(format!("{}{}", &self.url, &path).as_str()).call();
        if resp.status > 299 {
            return ExtensionResult::err("http request error");
        }
        let html = resp.body;

        let document = scraper::Html::parse_document(&html);

        let title = {
            let mut title = None;
            let selector = create_selector!("li[class=\"list-group-item d-none d-sm-block\"] h1");
            for element in document.select(&selector) {
                for text in element.text() {
                    if !text.is_empty() {
                        title = Some(text.to_string());
                        break;
                    }
                }
            }
            if let Some(title) = title {
                title
            } else {
                return ExtensionResult::err("no title");
            }
        };

        let description = {
            let mut desc = None;
            let selector = create_selector!("div[class=\"top-5 Content\"]");
            for element in document.select(&selector) {
                desc = element.text().next().map(str::to_string);
            }
            desc
        };

        let mut author = vec![];
        let selector = create_selector!("a[href^=\"/search/?author=\"]");
        for element in document.select(&selector) {
            for text in element.text() {
                author.push(text.to_string());
            }
        }

        let mut genre = vec![];
        let selector = create_selector!("a[href^=\"/search/?genre=\"]");
        for element in document.select(&selector) {
            for text in element.text() {
                genre.push(String::from(text));
            }
        }

        let status = {
            let selector = create_selector!("a[href^=\"/search/?status=\"]");

            document.select(&selector).next().and_then(|element| {
                element.value().attr("href").map(|h| {
                    h.strip_prefix("/search/?status=")
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| h.to_string())
                })
            })
        };

        let selector = create_selector!("img[class=\"img-fluid bottom-5\"]");

        let cover_url = document
            .select(&selector)
            .next()
            .and_then(|element| element.value().attr("src").map(str::to_string))
            .unwrap_or_default();

        ExtensionResult::ok(Manga {
            source_id: ID,
            title,
            description,
            author,
            genre,
            status,
            path,
            cover_url,
        })
    }

    fn get_chapters(&self, path: String) -> ExtensionResult<Vec<Chapter>> {
        let resp = Request::get(format!("{}{}", &self.url, &path).as_str()).call();
        if resp.status > 299 {
            return ExtensionResult::err("http request error");
        }
        let html = resp.body;

        let index_name = regex_find!(r#"(?<=vm\.IndexName = ").*(?=";)"#, html);

        let vm_dir = regex_find!(r#"(?<=vm\.Chapters = )\[.*\](?=;)"#, html);

        let ch_dirs: Vec<DirChapter> = match serde_json::from_str::<Vec<DirChapter>>(&vm_dir) {
            Ok(dirs) => dirs
                .iter()
                .map(|d| DirChapter {
                    index_name: index_name.to_string(),
                    ..d.clone()
                })
                .collect(),
            Err(e) => return ExtensionResult::err(format!("{}", e).as_str()),
        };

        let mut chapters = vec![];
        for ch in ch_dirs.iter() {
            let mut chapter = ch.chapter.clone();
            let t = chapter.remove(0);

            let index = if t != '1' {
                format!("-index-{}", t)
            } else {
                "".to_string()
            };

            chapter.insert(chapter.len() - 1, '.');
            let number = chapter.parse::<f64>().unwrap_or_default();

            chapters.push(Chapter {
                source_id: ID,
                title: format!("{} {}", ch.type_field, number.to_string()),
                path: format!(
                    "/read-online/{}-chapter-{}{}.html",
                    &index_name,
                    number.to_string(),
                    index,
                ),
                uploaded: ch.date,
                number: number + if index.is_empty() { 0.0 } else { 10000.0 },
                scanlator: "".to_string(),
            })
        }

        ExtensionResult::ok(chapters)
    }

    fn get_pages(&self, path: String) -> ExtensionResult<Vec<String>> {
        let resp = Request::get(format!("{}{}", &self.url, &path).as_str()).call();
        if resp.status > 299 {
            return ExtensionResult::err("http request error");
        }
        let html = resp.body;

        let index_name = regex_find!(r#"(?<=vm\.IndexName = ").*(?=";)"#, html);

        let cur_chapter = {
            let mat = regex_find!(r"(?<=vm\.CurChapter = ){.*}(?=;)", html);
            match serde_json::from_str::<CurChapter>(&mat) {
                Ok(cur_chater) => cur_chater,
                Err(e) => {
                    return ExtensionResult::err(
                        format!("failed to deserialize chapter: {}", e).as_str(),
                    );
                }
            }
        };

        let cur_path_name = regex_find!(r#"(?<=vm\.CurPathName = ").*(?=";)"#, html);

        let directory = {
            if cur_chapter.directory.is_empty() {
                "".to_string()
            } else {
                format!("{}/", cur_chapter.directory)
            }
        };
        let chapter_image = {
            let chapter = cur_chapter.chapter[1..cur_chapter.chapter.len() - 1].to_string();
            let odd = cur_chapter.chapter[cur_chapter.chapter.len() - 1..].to_string();
            if odd == "0" {
                chapter
            } else {
                format!("{}.{}", chapter, odd)
            }
        };

        let page = cur_chapter.page.parse::<i32>().unwrap_or(0);
        let mut pages = Vec::new();
        for i in 1..page + 1 {
            let page_image = {
                let s = format!("000{}", i);
                s[(s.len() - 3)..].to_string()
            };

            pages.push(format!(
                "https://{}/manga/{}/{}{}-{}.png",
                cur_path_name, index_name, directory, chapter_image, page_image
            ));
        }

        ExtensionResult::ok(pages)
    }
}
