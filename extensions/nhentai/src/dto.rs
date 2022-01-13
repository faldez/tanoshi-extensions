use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_number_from_string;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Results {
    pub result: Vec<Result>,
    #[serde(rename = "num_pages")]
    pub num_pages: i64,
    #[serde(rename = "per_page")]
    pub per_page: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: i64,
    #[serde(rename = "media_id")]
    pub media_id: String,
    pub title: Title,
    pub images: Images,
    pub scanlator: String,
    #[serde(rename = "upload_date")]
    pub upload_date: i64,
    pub tags: Vec<Tag>,
    #[serde(rename = "num_pages")]
    pub num_pages: i64,
    #[serde(rename = "num_favorites")]
    pub num_favorites: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title {
    pub english: String,
    pub japanese: Option<String>,
    pub pretty: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Images {
    pub pages: Vec<Page>,
    pub cover: Cover,
    pub thumbnail: Thumbnail,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub t: String,
    pub w: i64,
    pub h: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cover {
    pub t: String,
    pub w: i64,
    pub h: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail {
    pub t: String,
    pub w: i64,
    pub h: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: i64,
    #[serde(rename = "type")]
    pub type_field: String,
    pub name: String,
    pub url: String,
    pub count: i64,
}
