mod dto;

use std::collections::HashMap;

use anyhow::bail;
use dto::Result;
use tanoshi_lib::prelude::{
    ChapterInfo, Extension, Input, InputType, Lang, MangaInfo, PluginRegistrar,
};

use lazy_static::lazy_static;
use phf::phf_map;

use crate::dto::Results;

pub const ID: i64 = 9;
pub const NAME: &str = "nhentai";
pub const URL: &str = "https://nhentai.net";

tanoshi_lib::export_plugin!(register);

fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_function(Box::new(NHentai::default()));
}

static IMAGE_TYPE: phf::Map<&'static str, &'static str> = phf_map! {
    "j" => "jpg",
    "g" => "gif",
    "p" => "png",
};

lazy_static! {
    static ref TAG_FILTER: Input = Input::Text {
        name: "Tag".to_string(),
        state: None
    };
    static ref CHARACTERS_FILTER: Input = Input::Text {
        name: "Characters".to_string(),
        state: None
    };
    static ref ARTISTS_FILTER: Input = Input::Text {
        name: "Artists".to_string(),
        state: None
    };
    static ref GROUPS_FILTER: Input = Input::Text {
        name: "Groups".to_string(),
        state: None
    };
    static ref CATEGORIES_FILTER: Input = Input::Text {
        name: "Categories".to_string(),
        state: None
    };
    static ref PARODIES_FILTER: Input = Input::Text {
        name: "Parodies".to_string(),
        state: None
    };
    static ref SORT_FILTER: Input = Input::Select {
        name: "Sort".to_string(),
        values: vec![
            InputType::String("Popular".to_string()),
            InputType::String("Popular Week".to_string()),
            InputType::String("Popular Today".to_string()),
            InputType::String("Recent".to_string()),
        ],
        state: None
    };
    static ref FILTER_LIST: Vec<Input> = vec![
        TAG_FILTER.clone(),
        CHARACTERS_FILTER.clone(),
        CATEGORIES_FILTER.clone(),
        PARODIES_FILTER.clone(),
        ARTISTS_FILTER.clone(),
        GROUPS_FILTER.clone(),
        SORT_FILTER.clone()
    ];
    static ref LANGUAGE_SELECT: Input = Input::Select {
        name: "Language".to_string(),
        values: vec![
            InputType::String("Any".to_string()),
            InputType::String("English".to_string()),
            InputType::String("Japanese".to_string()),
            InputType::String("Chinese".to_string()),
        ],
        state: None
    };
    static ref BLACKLIST_TAG: Input = Input::Text {
        name: "Blacklist Tag".to_string(),
        state: None
    };
    static ref PREFERENCES: Vec<Input> = vec![LANGUAGE_SELECT.clone(), BLACKLIST_TAG.clone()];
}

pub struct NHentai {
    preferences: Vec<Input>,
}

impl Default for NHentai {
    fn default() -> Self {
        Self {
            preferences: PREFERENCES.clone(),
        }
    }
}

impl NHentai {
    fn query(&self, filters: Option<Vec<Input>>) -> String {
        let mut query = vec![];
        let mut sort = None;
        for pref in self.preferences.iter() {
            if LANGUAGE_SELECT.eq(pref) {
                if let Input::Select { state, values, .. } = pref {
                    if let Some(InputType::String(lang)) =
                        state.and_then(|index| values.get(index as usize))
                    {
                        if lang != "Any" {
                            query.push(format!("language:{}", lang.to_lowercase()));
                        }
                    }
                }
            } else if BLACKLIST_TAG.eq(pref) {
                if let Input::Text {
                    state: Some(state), ..
                } = pref
                {
                    for tag in state.split(',') {
                        query.push(format!("-tag:{}", tag.trim()))
                    }
                }
            }
        }

        if let Some(filters) = filters {
            for filter in filters.iter() {
                match filter {
                    Input::Text {
                        name,
                        state: Some(state),
                        ..
                    } if name == &TAG_FILTER.name() => {
                        for tag in state.split(',') {
                            if tag.starts_with('-') {
                                query.push(format!(
                                    "-{}:{}",
                                    name.to_lowercase(),
                                    tag.trim().replace("-", "")
                                ))
                            } else {
                                query.push(format!("{}:{}", name.to_lowercase(), tag.trim()))
                            }
                        }
                    }
                    Input::Text {
                        name,
                        state: Some(state),
                        ..
                    } => query.push(format!("{}:{}", name.to_lowercase(), state.trim())),
                    Input::Select {
                        name,
                        values,
                        state,
                        ..
                    } if name == &SORT_FILTER.name() => {
                        let state = state.unwrap_or(0);
                        if let Some(InputType::String(state)) = values.get(state as usize) {
                            sort = Some(format!("sort={}", state.replace(" ", "-").to_lowercase()));
                        }
                    }
                    _ => {}
                }
            }
        }

        let mut query_str = if query.is_empty() {
            r#""""#.to_string()
        } else {
            query.join(" ")
        };

        if let Some(sort) = sort {
            query_str = format!("{query_str}&{sort}");
        }

        query_str
    }
}

fn map_item_to_manga(item: &Result) -> MangaInfo {
    let mut tags = HashMap::new();
    for tag in item.tags.iter() {
        let entry = tags
            .entry(&tag.type_field)
            .or_insert_with(|| vec![tag.name.clone()]);
        entry.push(tag.name.clone());
    }

    let mut description = format!("#{}", item.id);
    if let Some(parody) = tags.get(&"parody".to_string()) {
        description = format!("{}\nParodies: {}", description, parody.join(","))
    }
    if let Some(character) = tags.get(&"character".to_string()) {
        description = format!("{}\nCharacters: {}", description, character.join(","))
    }
    if let Some(language) = tags.get(&"language".to_string()) {
        description = format!("{}\nLanguage: {}", description, language.join(","))
    }
    if let Some(category) = tags.get(&"category".to_string()) {
        description = format!("{}\nCategories: {}", description, category.join(","))
    }

    MangaInfo {
        source_id: ID,
        title: item.title.pretty.clone(),
        author: tags
            .get(&"artist".to_string())
            .cloned()
            .unwrap_or_else(Vec::new),
        genre: tags
            .get(&"tag".to_string())
            .cloned()
            .unwrap_or_else(Vec::new),
        status: None,
        description: Some(description),
        path: format!("/api/gallery/{}", item.id),
        cover_url: format!(
            "https://t.nhentai.net/galleries/{}/cover.{}",
            item.media_id,
            IMAGE_TYPE.get(&item.images.cover.t).unwrap()
        ),
    }
}

impl Extension for NHentai {
    fn get_source_info(&self) -> tanoshi_lib::prelude::SourceInfo {
        tanoshi_lib::prelude::SourceInfo {
            id: ID,
            name: "NHentai".to_string(),
            url: URL.to_string(),
            version: env!("CARGO_PKG_VERSION"),
            icon: "https://static.nhentai.net/img/logo.090da3be7b51.svg",
            languages: Lang::Multi(vec!["en".to_string(), "jp".to_string(), "ch".to_string()]),
            nsfw: true,
        }
    }

    fn set_preferences(
        &mut self,
        preferences: Vec<tanoshi_lib::prelude::Input>,
    ) -> anyhow::Result<()> {
        for input in preferences {
            for pref in self.preferences.iter_mut() {
                if input.eq(pref) {
                    *pref = input.clone();
                }
            }
        }

        Ok(())
    }

    fn get_popular_manga(&self, page: i64) -> anyhow::Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        let url = format!(
            "{}/api/galleries/search?query={}&page={}&sort=popular",
            URL,
            self.query(None),
            page
        );

        let res: Results = ureq::get(&url).call()?.into_json()?;
        Ok(res.result.iter().map(map_item_to_manga).collect())
    }

    fn get_latest_manga(&self, page: i64) -> anyhow::Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        let url = format!(
            "{}/api/galleries/search?query={}&page={}&sort=date",
            URL,
            self.query(None),
            page
        );

        let res: Results = ureq::get(&url).call()?.into_json()?;
        Ok(res.result.iter().map(map_item_to_manga).collect())
    }

    fn search_manga(
        &self,
        page: i64,
        query: Option<String>,
        filters: Option<Vec<tanoshi_lib::prelude::Input>>,
    ) -> anyhow::Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        let url = if let Some(query) = query {
            format!("{}/api/galleries/search?query={}&page={}", URL, query, page)
        } else if let Some(filters) = filters {
            format!(
                "{}/api/galleries/search?query={}&page={}",
                URL,
                self.query(Some(filters)),
                page
            )
        } else {
            bail!("query and filters can not be empty");
        };

        println!("{url}");

        let res: Results = ureq::get(&url).call()?.into_json()?;
        Ok(res.result.iter().map(map_item_to_manga).collect())
    }

    fn get_manga_detail(&self, path: String) -> anyhow::Result<tanoshi_lib::prelude::MangaInfo> {
        let url = format!("{}{}", URL, path);

        let res: dto::Result = ureq::get(&url).call()?.into_json()?;
        Ok(map_item_to_manga(&res))
    }

    fn get_chapters(&self, path: String) -> anyhow::Result<Vec<tanoshi_lib::prelude::ChapterInfo>> {
        let url = format!("{}{}", URL, path);

        let res: dto::Result = ureq::get(&url).call()?.into_json()?;
        Ok(vec![ChapterInfo {
            source_id: ID,
            title: "Chapter 1".to_string(),
            path,
            number: 1.0,
            scanlator: (!res.scanlator.is_empty()).then(|| res.scanlator),
            uploaded: res.upload_date,
        }])
    }

    fn get_pages(&self, path: String) -> anyhow::Result<Vec<String>> {
        let url = format!("{}{}", URL, path);

        let res: dto::Result = ureq::get(&url).call()?.into_json()?;

        Ok(res
            .images
            .pages
            .iter()
            .enumerate()
            .map(|(i, p)| {
                format!(
                    "https://i.nhentai.net/galleries/{}/{}.{}",
                    res.media_id,
                    i + 1,
                    IMAGE_TYPE.get(&p.t).unwrap()
                )
            })
            .collect())
    }

    fn headers(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }

    fn filter_list(&self) -> Vec<tanoshi_lib::prelude::Input> {
        FILTER_LIST.clone()
    }

    fn get_preferences(&self) -> anyhow::Result<Vec<tanoshi_lib::prelude::Input>> {
        Ok(self.preferences.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_popular_manga() {
        let nhentai = NHentai::default();
        let res = nhentai.get_popular_manga(1).unwrap();
        assert!(!res.is_empty());
    }

    #[test]
    fn test_get_latest_manga() {
        let nhentai = NHentai::default();
        let res = nhentai.get_latest_manga(1).unwrap();
        assert!(!res.is_empty());
    }

    #[test]
    fn test_search_manga() {
        let nhentai = NHentai::default();
        let res = nhentai
            .search_manga(1, Some("azur lane".to_string()), None)
            .unwrap();
        assert!(!res.is_empty());
    }

    #[test]
    fn test_search_manga_filter() {
        let nhentai = NHentai::default();
        let mut filters = nhentai.filter_list();
        for filter in filters.iter_mut() {
            if SORT_FILTER.eq(filter) {
                if let Input::Select { state, .. } = filter {
                    *state = Some(1);
                }
            } else if TAG_FILTER.eq(filter) {
                if let Input::Text { state, .. } = filter {
                    *state = Some("-big breasts".to_string());
                }
            } else if PARODIES_FILTER.eq(filter) {
                if let Input::Text { state, .. } = filter {
                    *state = Some("azur lane".to_string());
                }
            }
        }
        let res = nhentai.search_manga(1, None, Some(filters)).unwrap();
        assert!(!res.is_empty());
    }

    #[test]
    fn test_get_chapters() {
        let nhentai = NHentai::default();
        let res = nhentai
            .get_chapters("/api/gallery/385965".to_string())
            .unwrap();
        assert!(!res.is_empty());
    }

    #[test]
    fn test_get_pages() {
        let nhentai = NHentai::default();
        let res = nhentai
            .get_pages("/api/gallery/385965".to_string())
            .unwrap();
        assert!(!res.is_empty());
    }
}
