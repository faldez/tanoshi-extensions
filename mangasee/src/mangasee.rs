use std::fmt;

use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use rayon::prelude::*;
use serde::de::Deserializer;
use serde::de::{self, Unexpected};
use tanoshi_lib::extensions::Extension;
use tanoshi_lib::manga::{Chapter, Manga, Params, SortByParam, SortOrderParam, Source};

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dir {
    /// Link
    pub i: String,
    /// Title
    pub s: String,
    /// Official translation
    pub o: String,
    /// Scan status
    pub ss: String,
    /// Publish status
    pub ps: String,
    /// Type
    pub t: String,
    /// View ?
    pub v: String,
    /// vm
    pub vm: String,
    /// Year of published
    pub y: String,
    /// Authors
    pub a: Vec<String>,
    /// Alternative names
    pub al: Vec<String>,
    /// Latest
    pub l: String,
    /// Last chapter
    pub lt: i64,
    /// Last chapter
    #[serde(deserialize_with = "date_or_zero")]
    pub ls: NaiveDateTime,
    /// Genres
    pub g: Vec<String>,
    /// Hentai?
    pub h: bool,
}

struct DateOrZeroVisitor;

impl<'de> de::Visitor<'de> for DateOrZeroVisitor {
    type Value = NaiveDateTime;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer or a string")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(NaiveDateTime::from_timestamp(v as i64, 0))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if let Ok(dt) = NaiveDateTime::parse_from_str(v, "%Y-%m-%dT%H:%M:%S%z") {
            Ok(dt)
        } else {
            Err(E::invalid_value(Unexpected::Str(v), &self))
        }
    }
}

fn date_or_zero<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(DateOrZeroVisitor)
}

impl Into<Manga> for &Dir {
    fn into(self) -> Manga {
        Manga {
            id: 0,
            title: self.s.clone(),
            author: self.a.clone(),
            genre: self.g.clone(),
            status: Some(self.ss.clone()),
            description: None,
            path: format!("/manga/{}", self.i),
            thumbnail_url: format!("https://cover.mangabeast01.com/cover/{}.jpg", self.i),
            last_read: None,
            last_page: None,
            is_favorite: false,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirChapter {
    #[serde(skip)]
    pub index_name: String,
    #[serde(rename(deserialize = "Chapter"))]
    pub chapter: String,
    #[serde(rename(deserialize = "Type"))]
    pub type_field: String,
    #[serde(rename(deserialize = "Date"), deserialize_with = "parse_date")]
    pub date: NaiveDateTime,
    #[serde(rename(deserialize = "ChapterName"))]
    pub chapter_name: Option<String>,
    #[serde(rename(deserialize = "Page"))]
    pub page: Option<String>,
}

struct DateVisitor;

impl<'de> de::Visitor<'de> for DateVisitor {
    type Value = NaiveDateTime;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if let Ok(dt) = NaiveDateTime::parse_from_str(v, "%Y-%m-%d %H:%M:%S") {
            Ok(dt)
        } else {
            Err(E::invalid_value(Unexpected::Str(v), &self))
        }
    }
}

fn parse_date<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(DateVisitor)
}

impl Into<Chapter> for &DirChapter {
    fn into(self) -> Chapter {
        let mut chapter = self.chapter.clone();
        chapter.remove(0);
        chapter.insert_str(chapter.len() - 1, ".");
        let number = chapter.parse::<f32>().ok().map(|n| n.to_string());

        let mut ch = Chapter {
            id: 0,
            manga_id: 0,
            vol: None,
            no: None,
            title: None,
            url: format!(
                "/read-online/{}-chapter-{}.html",
                &self.index_name,
                number.clone().unwrap()
            ),
            read: None,
            uploaded: self.date,
        };

        if self.type_field == "Volume" {
            ch.vol = number
        } else {
            ch.no = number
        };

        ch
    }
}

pub struct Mangasee {}

impl Extension for Mangasee {
    fn info(&self) -> Source {
        Source {
            id: 0,
            name: "mangasee".to_string(),
            url: "https://mangasee123.com".to_string(),
            version: std::env!("PLUGIN_VERSION").to_string(),
        }
    }

    fn get_mangas(&self, url: &String, param: Params, _: String) -> Result<Vec<Manga>> {
        let base64_url = base64::encode(&url);
        let cache_path = dirs::home_dir()
            .unwrap()
            .join(".tanoshi")
            .join("cache")
            .join(base64_url);

        let vm_dir = match std::fs::read(&cache_path) {
            Ok(content) => String::from_utf8(content).unwrap(),
            Err(_) => {
                let resp = ureq::get(format!("{}/search", url).as_str()).call();
                let html = resp.into_string().unwrap();

                if let Some(i) = html.find("vm.Directory =") {
                    let dir = &html[i + 15..];
                    if let Some(i) = dir.find("}];") {
                        let vm_dir = &dir[..i + 2];
                        let _ = std::fs::write(&cache_path, &vm_dir);
                        vm_dir.to_string()
                    } else {
                        return Err(anyhow!("error get manga"));
                    }
                } else {
                    return Err(anyhow!("list not found"));
                }
            }
        };

        let mut dirs = match serde_json::from_str::<Vec<Dir>>(&vm_dir) {
            Ok(dirs) => dirs,
            Err(e) => return Err(anyhow!(e)),
        };

        let sort_by = param.sort_by.unwrap_or(SortByParam::Views);
        let sort_order = param.sort_order.unwrap_or(SortOrderParam::Asc);

        if let Some(keyword) = param.keyword {
            dirs.retain(|d| d.s.to_lowercase().contains(&keyword))
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
            SortByParam::Title => {}
        }

        let page = param
            .page
            .map(|p| p.parse::<usize>().ok().unwrap_or(1))
            .unwrap_or(1);
        let offset = (page - 1) * 20;
        let mangas = match dirs.len() {
            0..=20 => &dirs,
            _ => &dirs[offset..offset + 20],
        };

        return Ok(mangas.par_iter().map(|d| d.into()).collect());
    }

    /// Get the rest of details unreachable from `get_mangas`
    fn get_manga_info(&self, url: &String) -> Result<Manga> {
        let base64_url = base64::encode(&url);
        let cache_path = dirs::home_dir()
            .unwrap()
            .join(".tanoshi")
            .join("cache")
            .join(base64_url);
        let description = match std::fs::read(&cache_path) {
            Ok(content) => String::from_utf8(content).ok(),
            Err(_) => {
                let resp = ureq::get(url.as_str()).call();
                let html = resp.into_string().unwrap();

                let document = scraper::Html::parse_document(&html);

                let mut desc = None;
                let selector = scraper::Selector::parse("div[class=\"top-5 Content\"]").unwrap();
                for element in document.select(&selector) {
                    for text in element.text() {
                        desc = Some(String::from(text));
                    }
                }
                desc
            }
        };

        Ok(Manga {
            description,
            ..Default::default()
        })
    }

    fn get_chapters(&self, url: &String) -> Result<Vec<Chapter>> {
        let base64_url = base64::encode(format!("chapter:{}", &url));
        let cache_path = dirs::home_dir()
            .unwrap()
            .join(".tanoshi")
            .join("cache")
            .join(base64_url);
        let html = match std::fs::read(&cache_path) {
            Ok(content) => String::from_utf8(content).unwrap(),
            Err(_) => {
                let resp = ureq::get(url.as_str()).call();
                let html = resp.into_string().unwrap();
                if let Some(i) = html.find("vm.IndexName =") {
                    let dir = &html[i..];
                    if let Some(i) = dir.find("}];") {
                        let vm_dir = &dir[..i + 2];
                        let _ = std::fs::write(&cache_path, &vm_dir);
                        vm_dir.to_string()
                    } else {
                        return Err(anyhow!("error get chapters"));
                    }
                } else {
                    return Err(anyhow!("list not found"));
                }
            }
        };

        let index_name = if html.starts_with("vm.IndexName =") {
            let name = &html[15..];
            if let Some(i) = name.find(";") {
                &name[1..i - 1]
            } else {
                return Err(anyhow!("IndexName not found"));
            }
        } else {
            return Err(anyhow!("IndexName not found"));
        };

        let vm_dir = if let Some(i) = html.find("vm.Chapters =") {
            &html[i + 13..]
        } else {
            return Err(anyhow!("list not found"));
        };

        let ch_dirs: Vec<DirChapter> = match serde_json::from_str::<Vec<DirChapter>>(&vm_dir) {
            Ok(dirs) => dirs
                .par_iter()
                .map(|d| DirChapter {
                    index_name: index_name.to_string(),
                    ..d.clone()
                })
                .collect(),
            Err(e) => return Err(anyhow!(e)),
        };

        let chapters = ch_dirs.par_iter().map(|c| c.into()).collect();

        Ok(chapters)
    }

    fn get_pages(&self, url: &String) -> Result<Vec<String>> {
        let base64_url = base64::encode(&url);
        let cache_path = dirs::home_dir()
            .unwrap()
            .join(".tanoshi")
            .join("cache")
            .join(base64_url);
        let html = match std::fs::read(&cache_path) {
            Ok(content) => String::from_utf8(content).unwrap(),
            Err(_) => {
                let resp = ureq::get(url.as_str()).call();
                let html = resp.into_string().unwrap();

                if let Some(i) = html.find("vm.CurChapter = {") {
                    let dir = &html[i + 16..];
                    if let Some(i) = dir.find("vm.CHAPTERS = ") {
                        let vm_dir = &dir[..i];
                        let _ = std::fs::write(&cache_path, &vm_dir);
                        vm_dir.to_string()
                    } else {
                        return Err(anyhow!("error get pages"));
                    }
                } else {
                    return Err(anyhow!("list not found"));
                }
            }
        };

        let mut host: String = "".to_string();
        let mut ch: DirChapter = DirChapter {
            index_name: "".to_string(),
            chapter: "".to_string(),
            type_field: "".to_string(),
            date: NaiveDateTime::from_timestamp(0, 0),
            chapter_name: None,
            page: None,
        };

        if let Some(i) = html.find(";") {
            let ch_str = &html[..i];
            let path = &html[i..];
            ch = serde_json::from_str(ch_str).unwrap();
            if let Some(i) = path.find("vm.CurPathName = \"") {
                let path = &path[i + 18..];
                if let Some(i) = path.find("\";") {
                    host = String::from(&path[..i]);
                }
            }
        }

        let mut zeroes = "".to_string();
        for i in ch.chapter[1..ch.chapter.len() - 2].chars() {
            if i == '0' {
                zeroes.push_str("0");
            }
        }

        let page = ch.page.unwrap().parse::<i32>().unwrap() + 1;

        let mut pages = Vec::new();
        for i in 1..page {
            let mut page = url.clone();
            page = page.replace("mangasee123.com", &host);
            page = page.replace("read-online", "manga");
            page = page.replace("-chapter-", &format!("/{}", zeroes));
            page = page.replace(".html", "-");
            pages.push(format!("{}{:03}.png", page, i));
        }

        Ok(pages)
    }
}
