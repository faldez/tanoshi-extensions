use chrono::NaiveDateTime;
use serde::de::Deserializer;
use serde::de::{self, Unexpected};
use serde::Deserialize;
use tanoshi_lib::prelude::*;

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
    /// Views
    pub v: String,
    /// Views Montly
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

impl Dir {
    pub fn field_by_name(&self, name: &str) -> i64 {
        match name {
            "s" => self.s.parse().unwrap_or(0),
            "lt" => self.lt,
            "y" => self.y.parse().unwrap_or(0),
            "v" => self.v.parse().unwrap_or(0),
            "vm" => self.vm.parse().unwrap_or(0),
            _ => 0,
        }
    }
}

struct DateOrZeroVisitor;

impl<'de> de::Visitor<'de> for DateOrZeroVisitor {
    type Value = NaiveDateTime;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
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
            source_id: crate::ID,
            title: self.s.clone(),
            author: self.a.clone(),
            genre: self.g.clone(),
            status: Some(self.ss.clone()),
            description: None,
            path: format!("/manga/{}", self.i),
            cover_url: format!("https://cover.nep.li/cover/{}.jpg", self.i),
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

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CurChapter {
    pub chapter: String,
    #[serde(rename = "Type")]
    pub chapter_type: String,
    pub page: String,
    pub directory: String,
    #[serde(with = "date_format")]
    pub date: NaiveDateTime,
    pub chapter_name: Option<String>,
}

mod date_format {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer).unwrap();
        NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}
