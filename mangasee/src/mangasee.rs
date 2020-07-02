use anyhow::{anyhow, Result};
use serde_urlencoded;
use tanoshi_lib::extensions::Extension;
use tanoshi_lib::manga::{Chapter, Manga, Params, SortByParam, SortOrderParam, Source};

use chrono::NaiveDateTime;
use serde::de::Deserializer;
use serde::de::{self, Unexpected};
use std::fmt;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
        let resp = ureq::get(format!("{}/search", url).as_str()).call();

        let html = resp.into_string().unwrap();
        let mut dirs = if let Some(i) = html.find("vm.Directory =") {
            let dir = &html[i + 15..];
            if let Some(i) = dir.find("}];") {
                let vm_dir = &dir[..i + 2];
                match serde_json::from_str::<Vec<Dir>>(vm_dir) {
                    Ok(dirs) => dirs,
                    Err(e) => return Err(anyhow!(e)),
                }
            } else {
                return Err(anyhow!("error get manga"));
            }
        } else {
            return Err(anyhow!("error get manga"));
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

        return Ok(mangas.iter().map(|d| d.into()).collect());
    }

    /// Get the rest of details unreachable from `get_mangas`
    fn get_manga_info(&self, url: &String) -> Result<Manga> {
        let mut m = Manga::default();

        let resp = ureq::get(url.as_str()).call();
        let html = resp.into_string().unwrap();

        let document = scraper::Html::parse_document(&html);

        let selector = scraper::Selector::parse("body > div.container.MainContainer > div > div > div > div > div:nth-child(1) > div.col-md-9.col-sm-8.top-5 > ul > li:nth-child(10) > span").unwrap();
        for element in document.select(&selector) {
            for text in element.text() {
                m.description = Some(String::from(text));
            }
        }

        Ok(m)
    }

    fn get_chapters(&self, url: &String) -> Result<Vec<Chapter>> {
        let mut chapters: Vec<Chapter> = Vec::new();
        let resp = ureq::get(url.as_str()).call();
        let html = resp.into_string().unwrap();

        let document = scraper::Html::parse_document(&html);
        let selector = scraper::Selector::parse(".mainWell .chapter-list a[chapter]").unwrap();
        for element in document.select(&selector) {
            let mut chapter = Chapter::default();

            chapter.no = element.value().attr("chapter").map(|c| String::from(c));

            let link = element.value().attr("href").unwrap();
            chapter.url = link.replace("-page-1", "");

            let time_sel = scraper::Selector::parse("time[class*=\"SeriesTime\"]").unwrap();
            for time_el in element.select(&time_sel) {
                let date_str = time_el.value().attr("datetime").unwrap();
                chapter.uploaded =
                    chrono::NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S%:z")
                        .unwrap()
            }

            chapters.push(chapter);
        }

        Ok(chapters)
    }

    fn get_pages(&self, url: &String) -> Result<Vec<String>> {
        let mut pages = Vec::new();
        let resp = ureq::get(url.as_str()).call();
        let html = resp.into_string().unwrap();

        let document = scraper::Html::parse_document(&html);

        let selector = scraper::Selector::parse(".fullchapimage img").unwrap();
        for element in document.select(&selector) {
            pages.push(String::from(element.value().attr("src").unwrap()));
        }
        Ok(pages)
    }
}
