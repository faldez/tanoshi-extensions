use serde::{Deserialize, Serialize};

use self::manga::{
    AuthorAttributes, ChapterAttributes, CoverAttributes, MangaAttributes,
    ScanlationGroupAttributes, TagAttributes,
};

pub mod manga;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Relationship {
    Manga {
        id: String,
        attributes: Option<MangaAttributes>,
        #[serde(default)]
        relationships: Vec<Relationship>,
    },
    Chapter {
        id: String,
        attributes: Option<ChapterAttributes>,
        #[serde(default)]
        relationships: Vec<Relationship>,
    },
    CoverArt {
        id: String,
        attributes: Option<CoverAttributes>,
    },
    Author {
        id: String,
        attributes: Option<AuthorAttributes>,
    },
    Artist {
        id: String,
        attributes: Option<AuthorAttributes>,
    },
    ScanlationGroup {
        id: String,
        attributes: Option<ScanlationGroupAttributes>,
    },
    Tag {
        id: String,
        attributes: Option<TagAttributes>,
        #[serde(default)]
        relationships: Vec<Relationship>,
    },
    User {
        id: String,
    },
    CustomList {
        id: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Home {
    pub base_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Results {
    pub result: String,
    pub response: String,
    #[serde(flatten)]
    pub data: Data,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultsAtHome {
    pub result: String,
    pub base_url: String,
    pub chapter: AtHomeChapter,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AtHomeChapter {
    pub hash: String,
    pub data: Vec<String>,
    pub data_saver: Vec<String>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Data {
    Single {
        data: Relationship,
    },
    Multiple {
        data: Vec<Relationship>,
        limit: i64,
        offset: i64,
        total: i64,
    },
}
