use std::{collections::HashMap, fmt::Display};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;
use void::Void;

use super::Relationship;
use serde::de::{self, MapAccess, Visitor};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TagMode {
    And,
    Or,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ongoing,
    Completed,
    Hiatus,
    Cancelled,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Ongoing => write!(f, "ongoing"),
            Status::Completed => write!(f, "completed"),
            Status::Hiatus => write!(f, "hiatus"),
            Status::Cancelled => write!(f, "canceled"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Demographic {
    Shounen,
    Shoujo,
    Josei,
    Seinen,
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Rating {
    Safe,
    Suggestive,
    Erotica,
    Pornographic,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOrder {
    pub created_at: Option<Order>,
    pub updated_at: Option<Order>,
}

pub type Map = HashMap<String, String>;

fn sequence_or_map<'de, D>(deserializer: D) -> Result<Map, D::Error>
where
    D: Deserializer<'de>,
{
    // This is a Visitor that forwards string types to T's `FromStr` impl and
    // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
    // keep the compiler from complaining about T being an unused generic type
    // parameter. We need T in order to know the Value type for the Visitor
    // impl.
    struct SequenceOrMap<Map>(PhantomData<fn() -> Map>);

    impl<'de> Visitor<'de> for SequenceOrMap<Map> {
        type Value = Map;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("sequence or map")
        }

        fn visit_seq<A>(self, _seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            Ok(HashMap::new())
        }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            // `MapAccessDeserializer` is a wrapper that turns a `MapAccess`
            // into a `Deserializer`, allowing it to be used as the input to T's
            // `Deserialize` implementation. T then deserializes itself using
            // the entries from the map visitor.
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(SequenceOrMap(PhantomData))
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TagAttributes {
    #[serde(deserialize_with = "sequence_or_map")]
    pub name: Map,
    #[serde(deserialize_with = "sequence_or_map")]
    pub description: Map,
    pub group: String,
    pub version: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaAttributes {
    #[serde(deserialize_with = "sequence_or_map")]
    pub title: Map,
    pub alt_titles: Vec<Map>,
    #[serde(deserialize_with = "sequence_or_map")]
    pub description: Map,
    #[serde(default = "bool::default")]
    pub is_locked: bool,
    // pub links: Option<Map>,
    pub original_language: String,
    pub last_volume: Option<String>,
    pub last_chapter: Option<String>,
    pub publication_demographic: Option<Demographic>,
    pub status: Option<Status>,
    pub year: Option<i64>,
    pub content_rating: Rating,
    pub tags: Vec<Relationship>,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorAttributes {
    pub name: String,
    pub image_url: Option<String>,
    #[serde(deserialize_with = "sequence_or_map")]
    pub biography: Map,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverAttributes {
    pub volume: Option<String>,
    pub file_name: String,
    pub description: String,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanlationGroupAttributes {
    pub name: String,
    pub website: Option<String>,
    pub irc_server: Option<String>,
    pub irc_channel: Option<String>,
    pub discord: Option<String>,
    pub contact_email: Option<String>,
    pub description: Option<String>,
    pub locked: bool,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterAttributes {
    pub title: Option<String>,
    pub volume: Option<String>,
    pub chapter: Option<String>,
    pub translated_language: String,
    pub hash: String,
    pub data: Vec<String>,
    pub data_saver: Vec<String>,
    #[serde(default)]
    pub uploader: String,
    pub external_url: Option<String>,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub publish_at: DateTime<Utc>,
}

pub mod request {
    use super::*;

    #[derive(Debug, Clone, Default, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct MangaList {
        pub limit: i64,
        pub offset: i64,
        pub title: Option<String>,
        pub authors: Vec<String>,
        pub artists: Vec<String>,
        pub year: Option<i64>,
        pub included_tags: Vec<String>,
        pub included_tags_mode: Option<TagMode>,
        pub exluded_tags: Vec<String>,
        pub excluded_tags_mode: Option<TagMode>,
        pub status: Vec<Status>,
        pub original_language: Vec<String>,
        pub publication_demographic: Vec<Demographic>,
        pub ids: Vec<String>,
        pub content_rating: Vec<Rating>,
        pub created_at_since: Option<DateTime<Utc>>,
        pub updated_at_since: Option<DateTime<Utc>>,
        pub order: Option<ListOrder>,
        pub includes: Vec<String>,
    }

    #[derive(Debug, Clone, Default, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Manga {
        pub includes: Vec<String>,
    }

    #[derive(Debug, Clone, Default, Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct MangaFeed {
        pub limit: i64,
        pub offset: i64,
        pub translated_language: Vec<String>,
        pub created_at_since: Option<DateTime<Utc>>,
        pub updated_at_since: Option<DateTime<Utc>>,
        pub published_at_since: Option<DateTime<Utc>>,
        pub order: Option<ListOrder>,
        pub includes: Vec<String>,
    }
}
