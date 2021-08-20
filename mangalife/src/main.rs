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
        Mangalife {
            url: "https://manga4life.com".to_string(),
        }
    }
}

impl Mangalife {
    fn find_filter_map_value(
        filter_map: &Option<Filters>,
        key: &String,
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

        let mut dirs = serde_json::from_str::<Vec<Dir>>(&vm_dir).unwrap();

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
                    let v_a = a.v.parse::<i32>().unwrap();
                    let v_b = b.v.parse::<i32>().unwrap();
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
            let selector =
                scraper::Selector::parse("li[class=\"list-group-item d-none d-sm-block\"] h1")
                    .map_err(|e| format!("{:?}", e))
                    .unwrap();
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
            let selector = scraper::Selector::parse("div[class=\"top-5 Content\"]")
                .map_err(|e| format!("{:?}", e))
                .unwrap();
            for element in document.select(&selector) {
                for text in element.text() {
                    desc = Some(text.to_string());
                    break;
                }
            }
            desc
        };

        let mut author = vec![];
        let selector = scraper::Selector::parse("a[href^=\"/search/?author=\"]")
            .map_err(|e| format!("{:?}", e))
            .unwrap();
        for element in document.select(&selector) {
            for text in element.text() {
                author.push(text.to_string());
            }
        }

        let mut genre = vec![];
        let selector = scraper::Selector::parse("a[href^=\"/search/?genre=\"]")
            .map_err(|e| format!("{:?}", e))
            .unwrap();
        for element in document.select(&selector) {
            for text in element.text() {
                genre.push(String::from(text));
            }
        }

        let status = {
            let mut status = None;
            let selector = scraper::Selector::parse("a[href^=\"/search/?status=\"]")
                .map_err(|e| format!("{:?}", e))
                .unwrap();
            for element in document.select(&selector) {
                status = element.value().attr("href").map(|h| {
                    h.strip_prefix("/search/?status=")
                        .map(|s| s.to_string())
                        .unwrap_or(h.to_string())
                });
                break;
            }
            status
        };

        let mut cover_url = "".to_string();
        let selector = scraper::Selector::parse("img[class=\"img-fluid bottom-5\"]")
            .map_err(|e| format!("{:?}", e))
            .unwrap();
        for element in document.select(&selector) {
            cover_url = element
                .value()
                .attr("src")
                .map(|src| src.to_string())
                .ok_or(format!("no src"))
                .unwrap();
            break;
        }

        ExtensionResult::ok(Manga {
            source_id: ID,
            title,
            description,
            author,
            genre,
            status,
            path: path.clone(),
            cover_url,
        })
    }

    fn get_chapters(&self, path: String) -> ExtensionResult<Vec<Chapter>> {
        let resp = Request::get(format!("{}{}", &self.url, &path).as_str()).call();
        if resp.status > 299 {
            return ExtensionResult::err("http request error");
        }
        let html = resp.body;

        let index_name = {
            let mat = Regex::new(r#"(?<=vm\.IndexName = ").*(?=";)"#)
                .unwrap()
                .find(&html)
                .unwrap();
            mat.unwrap().as_str().to_string()
        };

        let vm_dir = {
            let mat = Regex::new(r#"(?<=vm\.Chapters = )\[.*\](?=;)"#)
                .unwrap()
                .find(&html)
                .unwrap();
            mat.unwrap().as_str().to_string()
        };

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

            chapter.insert_str(chapter.len() - 1, ".");
            let number = chapter.parse::<f64>().unwrap();

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
                number: number + if index == "" { 0.0 } else { 10000.0 },
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

        let index_name = {
            let mat = Regex::new(r#"(?<=vm\.IndexName = ").*(?=";)"#)
                .unwrap()
                .find(&html)
                .unwrap();
            mat.unwrap().as_str().to_string()
        };

        let cur_chapter = {
            let mat = Regex::new(r"(?<=vm\.CurChapter = ){.*}(?=;)")
                .unwrap()
                .find(&html)
                .unwrap();
            let cur_chapter_str = mat.unwrap().as_str();
            serde_json::from_str::<CurChapter>(cur_chapter_str).unwrap()
        };

        let cur_path_name = {
            let mat = Regex::new(r#"(?<=vm\.CurPathName = ").*(?=";)"#)
                .unwrap()
                .find(&html)
                .unwrap();
            mat.unwrap().as_str().to_string()
        };

        let directory = {
            if cur_chapter.directory == "" {
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
