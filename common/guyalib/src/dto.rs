use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Detail {
    pub author: String,
    pub artist: String,
    pub description: String,
    pub slug: String,
    pub cover: String,
    pub groups: HashMap<String, String>,
    pub last_updated: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Series {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub artist: String,
    pub groups: HashMap<String, String>,
    pub cover: String,
    pub preferred_sort: Vec<String>,
    pub chapters: HashMap<String, Chapter>,
    pub next_release_page: bool,
    pub next_release_time: f64,
    pub next_release_html: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    pub volume: String,
    pub title: String,
    pub folder: String,
    pub groups: HashMap<String, Vec<String>>,
    pub release_date: HashMap<String, f64>,
}
