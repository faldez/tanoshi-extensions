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
    pub data: Vec<Relationship>,
    pub limit: i64,
    pub offset: i64,
    pub total: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SingleResult {
    pub result: String,
    pub response: String,
    pub data: Relationship,
}
