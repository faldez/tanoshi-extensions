use std::fmt;

use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use rayon::prelude::*;
use serde::de::Deserializer;
use serde::de::{self, Unexpected};
use serde::Deserialize;
use fancy_regex::Regex;
use tanoshi_lib::extensions::Extension;
use tanoshi_lib::manga::{Chapter, Manga, Params, SortByParam, SortOrderParam, Source};

pub static NAME: &str = "mangasee";

#[derive(Debug, Clone, PartialEq, Deserialize)]
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
            source: NAME.to_string(),
            title: self.s.clone(),
            author: self.a.clone(),
            genre: self.g.clone(),
            status: Some(self.ss.clone()),
            description: None,
            path: format!("/manga/{}", self.i),
            thumbnail_url: format!("https://cover.nep.li/cover/{}.jpg", self.i),
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
            source: NAME.to_string(),
            manga_id: 0,
            vol: None,
            no: None,
            title: None,
            path: format!(
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

#[derive(Debug, Deserialize)]
#[serde(rename_all="PascalCase")]
pub struct CurChapter {
    pub chapter: String,
    #[serde(rename = "Type")]
    pub chapter_type: String,
    pub page: String,
    pub directory: String,
    #[serde(with ="date_format")]
    pub date: NaiveDateTime,
    pub chapter_name: Option<String>
}

mod date_format {
    use chrono::{NaiveDateTime};
    use serde::{self, Deserialize, Serializer, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(
        date: &NaiveDateTime,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

pub struct Mangasee {
    url: String,
}

impl Mangasee {
    pub fn new() -> Mangasee {
        Mangasee {
            url: "https://mangasee123.com".to_string(),
        }
    }
}

impl Extension for Mangasee {
    fn info(&self) -> Source {
        Source {
            name: NAME.to_string(),
            url: self.url.clone(),
            version: std::env!("PLUGIN_VERSION").to_string(),
        }
    }

    fn get_mangas(&self, param: Params, _: String) -> Result<Vec<Manga>> {
        let url = format!("{}/search", &self.url);
        let vm_dir = {
            let resp = ureq::get(&url).call();
            let html = resp.into_string().unwrap();

            if let Some(i) = html.find("vm.Directory =") {
                let dir = &html[i + 15..];
                if let Some(i) = dir.find("}];") {
                    let vm_dir = &dir[..i + 2];
                    vm_dir.to_string()
                } else {
                    return Err(anyhow!("error get manga"));
                }
            } else {
                return Err(anyhow!("list not found"));
            }
        };

        let mut dirs = match serde_json::from_str::<Vec<Dir>>(&vm_dir) {
            Ok(dirs) => dirs,
            Err(e) => return Err(anyhow!(e)),
        };

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
    fn get_manga_info(&self, path: &String) -> Result<Manga> {
        let url = format!("{}{}", &self.url, &path);
        let description = {
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
        };

        Ok(Manga {
            description,
            ..Default::default()
        })
    }

    fn get_chapters(&self, path: &String) -> Result<Vec<Chapter>> {
        let url = format!("{}{}", &self.url, &path);
        let html = {
            let resp = ureq::get(url.as_str()).call();
            let html = resp.into_string().unwrap();
            if let Some(i) = html.find("vm.IndexName =") {
                let dir = &html[i..];
                if let Some(i) = dir.find("}];") {
                    let vm_dir = &dir[..i + 2];
                    vm_dir.to_string()
                } else {
                    return Err(anyhow!("error get chapters"));
                }
            } else {
                return Err(anyhow!("list not found"));
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

    fn get_pages(&self, path: &String) -> Result<Vec<String>> {
        let url = format!("{}{}", &self.url, &path);
        let resp = ureq::get(url.as_str()).call();
        let html = resp.into_string().unwrap();

        let index_name = {
            let mat = Regex::new(r#"(?<=vm\.IndexName = ").*(?=";)"#).unwrap().find(&html).unwrap();
            mat.unwrap().as_str().to_string()
        };

        let cur_chapter = {
            let mat = Regex::new(r"(?<=vm\.CurChapter = ){.*}(?=;)").unwrap().find(&html).unwrap();
            let cur_chapter_str = mat.unwrap().as_str();
            serde_json::from_str::<CurChapter>(cur_chapter_str).unwrap()
        };

        let cur_path_name = {
            let mat = Regex::new(r#"(?<=vm\.CurPathName = ").*(?=";)"#).unwrap().find(&html).unwrap();
            mat.unwrap().as_str().to_string()
        };
        
        // https://{{vm.CurPathName}}/manga/Sono-Bisque-Doll-Wa-Koi-Wo-Suru/{{vm.CurChapter.Directory == '' ? '' : vm.CurChapter.Directory+'/'}}{{vm.ChapterImage(vm.CurChapter.Chapter)}}-{{vm.PageImage(Page)}}.png
        let directory = {
            if cur_chapter.directory == "" {
                "".to_string()
            } else {
                format!("{}/", cur_chapter.directory)
            }
        };
        let chapter_image = {
            /*
            vm.ChapterImage = function(ChapterString){
                var Chapter = ChapterString.slice(1,-1);
                var Odd = ChapterString[ChapterString.length -1];
                if(Odd == 0){
                    return Chapter;
                }else{
                    return Chapter + "." + Odd;
                }
            };
            */
            let chapter = cur_chapter.chapter[1..cur_chapter.chapter.len()-1].to_string();
            let odd = cur_chapter.chapter[cur_chapter.chapter.len()-1..].to_string();
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
                /*
                vm.PageImage = function(PageString){
			    	var s = "000" + PageString;
			    	return s.substr(s.length - 3);
			    }
                */
                let s = format!("000{}", i);
                s[(s.len() - 3)..].to_string()
            };

            pages.push(format!("https://{}/manga/{}/{}{}-{}.png", cur_path_name, index_name, directory, chapter_image, page_image));
        }

        Ok(pages)
    }
}
