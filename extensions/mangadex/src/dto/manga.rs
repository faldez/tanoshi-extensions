use std::{collections::HashMap, fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use std::marker::PhantomData;

use super::Relationship;
use serde::de::{self, MapAccess, Visitor};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TagMode {
    And,
    Or,
}

impl FromStr for TagMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "AND" => Ok(TagMode::And),
            "OR" => Ok(TagMode::Or),
            _ => Err(anyhow::anyhow!("no such status")),
        }
    }
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

impl FromStr for Status {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ongoing" => Ok(Status::Ongoing),
            "completed" => Ok(Status::Completed),
            "hiatus" => Ok(Status::Hiatus),
            "canceled" => Ok(Status::Cancelled),
            _ => Err(anyhow::anyhow!("no such status")),
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

impl Display for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rating::Safe => write!(f, "safe"),
            Rating::Suggestive => write!(f, "suggestive"),
            Rating::Erotica => write!(f, "erotica"),
            Rating::Pornographic => write!(f, "pornographic"),
        }
    }
}

impl FromStr for Rating {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "safe" => Ok(Rating::Safe),
            "suggestive" => Ok(Rating::Suggestive),
            "erotica" => Ok(Rating::Erotica),
            "pornographic" => Ok(Rating::Pornographic),
            _ => Err(anyhow::anyhow!("no such rating")),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOrder {
    pub created_at: Option<Order>,
    pub updated_at: Option<Order>,
    pub followed_count: Option<Order>,
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
    pub external_url: Option<String>,
    pub version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub publish_at: DateTime<Utc>,
    pub pages: i64,
}

pub mod request {
    use tanoshi_lib::prelude::{Input, InputType, TriState};

    use crate::filter::*;

    use super::*;

    #[derive(Debug, Clone, Deserialize, Serialize)]
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
        pub excluded_tags: Vec<String>,
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

    impl Default for MangaList {
        fn default() -> Self {
            Self {
                includes: vec![
                    "cover_art".to_string(),
                    "author".to_string(),
                    "artist".to_string(),
                    "scanlation_group".to_string(),
                ],
                limit: Default::default(),
                offset: Default::default(),
                title: Default::default(),
                authors: Default::default(),
                artists: Default::default(),
                year: Default::default(),
                included_tags: Default::default(),
                included_tags_mode: Default::default(),
                excluded_tags: Default::default(),
                excluded_tags_mode: Default::default(),
                status: Default::default(),
                original_language: Default::default(),
                publication_demographic: Default::default(),
                ids: Default::default(),
                content_rating: Default::default(),
                created_at_since: Default::default(),
                updated_at_since: Default::default(),
                order: Default::default(),
            }
        }
    }

    impl MangaList {
        pub fn to_query_string(&self) -> anyhow::Result<String> {
            Ok(serde_qs::to_string(self)?)
        }
    }

    impl From<Vec<Input>> for MangaList {
        fn from(filters: Vec<Input>) -> Self {
            let mut included_tags = vec![];
            let mut included_tags_mode = None;
            let mut excluded_tags = vec![];
            let mut excluded_tags_mode = None;
            let mut status = vec![];
            let mut content_rating = vec![];
            let mut title = None;
            let mut year = None;
            let mut artists = vec![];
            let mut authors = vec![];

            for filter in filters {
                if TITLE_FILTER.eq(&filter) {
                    if let Input::Text { state, .. } = filter {
                        title = state;
                    }
                } else if AUTHOR_FILTER.eq(&filter) {
                    if let Input::Text { state, .. } = filter {
                        authors = state
                            .map(|s| s.split(',').map(|s| s.to_string()).collect())
                            .unwrap_or_default();
                    }
                } else if ARTIST_FILTER.eq(&filter) {
                    if let Input::Text { state, .. } = filter {
                        artists = state
                            .map(|s| s.split(',').map(|s| s.to_string()).collect())
                            .unwrap_or_default();
                    }
                } else if YEAR_FILTER.eq(&filter) {
                    if let Input::Text { state, .. } = filter {
                        year = state.and_then(|y| y.parse().ok());
                    }
                } else if TAG_FILTERS.eq(&filter) {
                    if let Input::Group { state, .. } = filter {
                        included_tags = state
                            .iter()
                            .filter_map(|s| {
                                if let Input::State { name, selected } = s {
                                    (selected.unwrap_or_default() == TriState::Included)
                                        .then(|| name.clone())
                                        .and_then(|name| {
                                            TAG_ID_MAP.get(&name).map(|id| id.to_string())
                                        })
                                } else {
                                    None
                                }
                            })
                            .collect();

                        excluded_tags = state
                            .iter()
                            .filter_map(|s| {
                                if let Input::State { name, selected } = s {
                                    (selected.unwrap_or_default() == TriState::Excluded)
                                        .then(|| name.clone())
                                        .and_then(|name| {
                                            TAG_ID_MAP.get(&name).map(|id| id.to_string())
                                        })
                                } else {
                                    None
                                }
                            })
                            .collect();
                    }
                } else if INCLUDED_TAGS_MODE.eq(&filter) {
                    if let Input::Select { values, state, .. } = filter {
                        included_tags_mode =
                            state.and_then(|i| values.get(i as usize)).and_then(|m| {
                                if let InputType::String(mode) = m {
                                    TagMode::from_str(mode).ok()
                                } else {
                                    None
                                }
                            });
                    }
                } else if EXCLUDED_TAGS_MODE.eq(&filter) {
                    if let Input::Select { values, state, .. } = filter {
                        excluded_tags_mode =
                            state.and_then(|i| values.get(i as usize)).and_then(|m| {
                                if let InputType::String(mode) = m {
                                    TagMode::from_str(mode).ok()
                                } else {
                                    None
                                }
                            });
                    }
                } else if STATUS_FILTER.eq(&filter) {
                    if let Input::Group { state, .. } = filter {
                        status = state
                            .iter()
                            .filter_map(|input| {
                                if let Input::Checkbox { name, state } = input {
                                    if state.unwrap_or(false) {
                                        Status::from_str(name).ok()
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect();
                    }
                } else if CONTENT_RATING_FILTER.eq(&filter) {
                    if let Input::Group { state, .. } = filter {
                        content_rating = state
                            .iter()
                            .filter_map(|input| {
                                if let Input::Checkbox { name, state } = input {
                                    if state.unwrap_or(false) {
                                        Rating::from_str(name).ok()
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect();
                    }
                }
            }

            Self {
                title,
                authors,
                artists,
                year,
                included_tags,
                included_tags_mode,
                excluded_tags,
                excluded_tags_mode,
                status,
                content_rating,
                ..Default::default()
            }
        }
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

#[cfg(test)]
mod test {
    use tanoshi_lib::prelude::Input;

    use super::request::MangaList;

    #[test]
    fn test_input_to_manga_list_request() {
        let input = vec![
            Input::Group {
                name: "Status".to_string(),
                state: vec![
                    Input::Checkbox {
                        name: "ongoing".to_string(),
                        state: Some(true),
                    },
                    Input::Checkbox {
                        name: "completed".to_string(),
                        state: Some(true),
                    },
                    Input::Checkbox {
                        name: "hiatus".to_string(),
                        state: Some(true),
                    },
                    Input::Checkbox {
                        name: "canceled".to_string(),
                        state: Some(true),
                    },
                ],
            },
            Input::Group {
                name: "Content Rating".to_string(),
                state: vec![
                    Input::Checkbox {
                        name: "safe".to_string(),
                        state: Some(true),
                    },
                    Input::Checkbox {
                        name: "suggestive".to_string(),
                        state: Some(true),
                    },
                    Input::Checkbox {
                        name: "erotica".to_string(),
                        state: Some(true),
                    },
                    Input::Checkbox {
                        name: "pornographic".to_string(),
                        state: Some(true),
                    },
                ],
            },
        ];

        let manga_list: MangaList = input.into();
        let query = manga_list.to_query_string().unwrap();
        assert_eq!("limit=0&offset=0&status[0]=ongoing&status[1]=completed&status[2]=hiatus&status[3]=cancelled&contentRating[0]=safe&contentRating[1]=suggestive&contentRating[2]=erotica&contentRating[3]=pornographic&includes[0]=cover_art&includes[1]=author&includes[2]=artist&includes[3]=scanlation_group", query, "expected got {query}");
    }
}
