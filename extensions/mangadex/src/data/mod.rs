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
    },
    Chapter {
        id: String,
        attributes: Option<ChapterAttributes>,
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
pub struct Result {
    pub result: String,
    pub data: Relationship,
    pub relationships: Vec<Relationship>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Results {
    pub results: Vec<Result>,
    pub limit: i64,
    pub offset: i64,
    pub total: i64,
}
