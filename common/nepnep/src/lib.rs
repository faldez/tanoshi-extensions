mod dto;

use std::collections::HashSet;

use anyhow::{anyhow, bail, Result};
use fancy_regex::Regex;
use scraper::{Html, Selector};
use tanoshi_lib::prelude::{ChapterInfo, Input, InputType, MangaInfo, TriState};

use crate::dto::{CurChapter, Dir, DirChapter};

use lazy_static::lazy_static;

lazy_static! {
    static ref KEYWORD_FILTER: Input = Input::Text {
        name: "Series Name".to_string(),
        state: None
    };
    static ref GENRE_FILTER: Input = Input::Group {
        name: "Genres".to_string(),
        state: vec![
            Input::State {
                name: "Action".to_string(),
                selected: None
            },
            Input::State {
                name: "Adult".to_string(),
                selected: None
            },
            Input::State {
                name: "Adventure".to_string(),
                selected: None
            },
            Input::State {
                name: "Comedy".to_string(),
                selected: None
            },
            Input::State {
                name: "Doujinshi".to_string(),
                selected: None
            },
            Input::State {
                name: "Drama".to_string(),
                selected: None
            },
            Input::State {
                name: "Ecchi".to_string(),
                selected: None
            },
            Input::State {
                name: "Fantasy".to_string(),
                selected: None
            },
            Input::State {
                name: "Gender Bender".to_string(),
                selected: None
            },
            Input::State {
                name: "Harem".to_string(),
                selected: None
            },
            Input::State {
                name: "Hentai".to_string(),
                selected: None
            },
            Input::State {
                name: "Historical".to_string(),
                selected: None
            },
            Input::State {
                name: "Horror".to_string(),
                selected: None
            },
            Input::State {
                name: "Isekai".to_string(),
                selected: None
            },
            Input::State {
                name: "Josei".to_string(),
                selected: None
            },
            Input::State {
                name: "Lolicon".to_string(),
                selected: None
            },
            Input::State {
                name: "Martial Arts".to_string(),
                selected: None
            },
            Input::State {
                name: "Mature".to_string(),
                selected: None
            },
            Input::State {
                name: "Mecha".to_string(),
                selected: None
            },
            Input::State {
                name: "Mystery".to_string(),
                selected: None
            },
            Input::State {
                name: "Psychological".to_string(),
                selected: None
            },
            Input::State {
                name: "Romance".to_string(),
                selected: None
            },
            Input::State {
                name: "School Life".to_string(),
                selected: None
            },
            Input::State {
                name: "Sci-fi".to_string(),
                selected: None
            },
            Input::State {
                name: "Seinen".to_string(),
                selected: None
            },
            Input::State {
                name: "Shotacon".to_string(),
                selected: None
            },
            Input::State {
                name: "Shoujo".to_string(),
                selected: None
            },
            Input::State {
                name: "Shoujo Ai".to_string(),
                selected: None
            },
            Input::State {
                name: "Shounen".to_string(),
                selected: None
            },
            Input::State {
                name: "Shounen Ai".to_string(),
                selected: None
            },
            Input::State {
                name: "Slice of Life".to_string(),
                selected: None
            },
            Input::State {
                name: "Smut".to_string(),
                selected: None
            },
            Input::State {
                name: "Sports".to_string(),
                selected: None
            },
            Input::State {
                name: "Supernatural".to_string(),
                selected: None
            },
            Input::State {
                name: "Tragedy".to_string(),
                selected: None
            },
            Input::State {
                name: "Yaoi".to_string(),
                selected: None
            },
            Input::State {
                name: "Yuri".to_string(),
                selected: None
            },
        ]
    };
    static ref SCAN_STATUS_FILTER: Input = Input::Select {
        name: "Scan Status".to_string(),
        values: vec![
            InputType::String("Any".to_string()),
            InputType::String("Cancelled".to_string()),
            InputType::String("Complete".to_string()),
            InputType::String("Discontinued".to_string()),
            InputType::String("Hiatus".to_string()),
            InputType::String("Ongoing".to_string())
        ],
        state: None
    };
    static ref PUBLISH_STATUS_FILTER: Input = Input::Select {
        name: "Publish Status".to_string(),
        values: vec![
            InputType::String("Any".to_string()),
            InputType::String("Cancelled".to_string()),
            InputType::String("Complete".to_string()),
            InputType::String("Discontinued".to_string()),
            InputType::String("Hiatus".to_string()),
            InputType::String("Ongoing".to_string())
        ],
        state: None
    };
    static ref SORT_BY_FILTER: Input = Input::Sort {
        name: "Sort By".to_string(),
        values: vec![
            InputType::String("Alphabetical".to_string()),
            InputType::String("Year Released".to_string()),
            InputType::String("Popular".to_string()),
        ],
        selection: None
    };
    static ref FILTER_LIST: Vec<Input> = vec![
        KEYWORD_FILTER.clone(),
        GENRE_FILTER.clone(),
        SCAN_STATUS_FILTER.clone(),
        PUBLISH_STATUS_FILTER.clone(),
        SORT_BY_FILTER.clone()
    ];
}

pub fn get_filter_list() -> Vec<Input> {
    FILTER_LIST.clone()
}

pub fn get_all_manga(url: &str) -> Result<Vec<Dir>> {
    let html = ureq::get(&format!("{}/search", url))
        .call()?
        .into_string()?;
    let start_index = html
        .find("vm.Directory =")
        .ok_or_else(|| anyhow!("vm.Directory not found"))?;
    let dir = &html[start_index + 15..];
    let end_index = dir
        .find("}];")
        .ok_or_else(|| anyhow!("vm.Directory not found"))?;
    let vm_dir = dir[..end_index + 2].to_string();
    Ok(serde_json::from_str::<Vec<Dir>>(&vm_dir)?)
}

fn sort_popular(dirs: &mut Vec<Dir>, asc: bool) {
    dirs.sort_by(|a, b| {
        let v_a = a.v.parse::<i32>().unwrap_or_default();
        let v_b = b.v.parse::<i32>().unwrap_or_default();
        if asc {
            v_a.cmp(&v_b)
        } else {
            v_b.cmp(&v_a)
        }
    });
}

fn sort_latest(dirs: &mut Vec<Dir>, asc: bool) {
    dirs.sort_by(|a, b| {
        if asc {
            a.lt.cmp(&b.lt)
        } else {
            b.lt.cmp(&a.lt)
        }
    });
}

fn sort_alphabetically(dirs: &mut Vec<Dir>, asc: bool) {
    dirs.sort_by(|a, b| if asc { a.s.cmp(&b.s) } else { b.s.cmp(&a.s) });
}

fn sort_year_released(dirs: &mut Vec<Dir>, asc: bool) {
    dirs.sort_by(|a, b| {
        let y_a = a.y.parse::<i32>().unwrap_or_default();
        let y_b = b.y.parse::<i32>().unwrap_or_default();
        if asc {
            y_a.cmp(&y_b)
        } else {
            y_b.cmp(&y_a)
        }
    });
}

pub fn get_popular_manga(source_id: i64, url: &str, mut page: i64) -> Result<Vec<MangaInfo>> {
    if page < 1 {
        page = 1;
    }
    let offset = (page - 1) * 20;
    let mut dirs = get_all_manga(url)?;
    sort_popular(&mut dirs, false);

    let manga = dirs
        .iter()
        .skip(offset as usize)
        .take(20)
        .map(|dir| {
            let mut manga: MangaInfo = dir.into();
            manga.source_id = source_id;
            manga
        })
        .collect();
    Ok(manga)
}

pub fn get_latest_manga(source_id: i64, url: &str, mut page: i64) -> Result<Vec<MangaInfo>> {
    if page < 1 {
        page = 1;
    }
    let offset = (page - 1) * 20;
    let mut dirs = get_all_manga(url)?;
    sort_latest(&mut dirs, false);

    let manga = dirs
        .iter()
        .skip(offset as usize)
        .take(20)
        .map(|dir| {
            let mut manga: MangaInfo = dir.into();
            manga.source_id = source_id;
            manga
        })
        .collect();
    Ok(manga)
}

fn filter_genre(dirs: &mut Vec<Dir>, genres: &[Input]) {
    let included_genres: HashSet<String> = genres
        .iter()
        .filter_map(|input| {
            if let Input::State { name, selected } = input {
                (selected.unwrap_or_default() == TriState::Included).then(|| name.clone())
            } else {
                None
            }
        })
        .collect();

    if !included_genres.is_empty() {
        dirs.retain(|dir| {
            let mut has = 0;
            for g in dir.g.iter() {
                if included_genres.contains(g) {
                    has += 1
                }
            }

            has == included_genres.len() as i32
        });
    }

    let excluded_genres: HashSet<String> = genres
        .iter()
        .filter_map(|input| {
            if let Input::State { name, selected } = input {
                (selected.unwrap_or_default() == TriState::Excluded).then(|| name.clone())
            } else {
                None
            }
        })
        .collect();

    if !excluded_genres.is_empty() {
        dirs.retain(|dir| {
            for g in dir.g.iter() {
                if excluded_genres.contains(g) {
                    return false;
                }
            }

            true
        });
    }
}

fn filter_keyword(dirs: &mut Vec<Dir>, keyword: &str) {
    dirs.retain(|dir| dir.s.to_lowercase().contains(&keyword.to_lowercase()))
}

pub fn search_manga(
    source_id: i64,
    url: &str,
    mut page: i64,
    query: Option<String>,
    filters: Option<Vec<Input>>,
) -> Result<Vec<MangaInfo>> {
    if query.is_none() && filters.is_none() {
        bail!("query and filters cannot be both empty")
    }

    if page < 1 {
        page = 1;
    }

    let offset = (page - 1) * 20;
    let mut dirs = get_all_manga(url)?;

    if let Some(filters) = filters {
        for filter in filters {
            println!("filter: {:?}", filter);
            if KEYWORD_FILTER.eq(&filter) {
                if let Input::Text {
                    state: Some(state), ..
                } = filter
                {
                    filter_keyword(&mut dirs, &state);
                }
            } else if GENRE_FILTER.eq(&filter) {
                if let Input::Group { state, .. } = filter {
                    if !state.is_empty() {
                        filter_genre(&mut dirs, &state);
                    }
                }
            // } else if SCAN_STATUS_FILTER.eq(&filter) {
            // } else if PUBLISH_STATUS_FILTER.eq(&filter) {
            } else if SORT_BY_FILTER.eq(&filter) {
                if let Input::Sort { selection, .. } = filter {
                    let selection = selection.unwrap_or((0, false));
                    match selection {
                        (0, asc) => sort_alphabetically(&mut dirs, asc),
                        (1, asc) => sort_year_released(&mut dirs, asc),
                        (2, asc) => sort_popular(&mut dirs, asc),
                        _ => {}
                    }
                }
            }
        }
    }

    let manga = dirs
        .iter()
        .skip(offset as usize)
        .take(20)
        .map(|dir| {
            let mut manga: MangaInfo = dir.into();
            manga.source_id = source_id;
            manga
        })
        .collect();
    Ok(manga)
}

pub fn get_manga_detail(source_id: i64, url: &str, path: String) -> Result<MangaInfo> {
    let body = ureq::get(&format!("{}{}", url, path))
        .call()?
        .into_string()?;
    let doc = Html::parse_document(&body);

    let title = doc
        .select(
            &Selector::parse("li[class=\"list-group-item d-none d-sm-block\"] h1")
                .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?,
        )
        .next()
        .and_then(|el| el.text().next().map(|e| e.to_string()))
        .unwrap_or_default();
    let description = doc
        .select(
            &Selector::parse("div[class=\"top-5 Content\"]")
                .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?,
        )
        .next()
        .and_then(|el| el.text().next().map(|e| e.to_string()));
    let author = doc
        .select(
            &Selector::parse("a[href^=\"/search/?author=\"]")
                .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?,
        )
        .next()
        .map(|el| el.text().into_iter().map(|s| s.to_string()).collect())
        .unwrap_or_else(Vec::new);
    let genre = doc
        .select(
            &Selector::parse("a[href^=\"/search/?genre=\"]")
                .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?,
        )
        .next()
        .map(|el| el.text().into_iter().map(|s| s.to_string()).collect())
        .unwrap_or_else(Vec::new);
    let status = doc
        .select(
            &Selector::parse("a[href^=\"/search/?status=\"]")
                .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?,
        )
        .next()
        .and_then(|el| el.text().next().map(|s| s.replace("/search/?status=", "")));
    let cover_url = doc
        .select(
            &Selector::parse("img[class=\"img-fluid bottom-5\"]")
                .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?,
        )
        .next()
        .and_then(|el| el.value().attr("src").map(str::to_string))
        .unwrap_or_default();

    Ok(MangaInfo {
        source_id,
        title,
        author,
        genre,
        status,
        description,
        path,
        cover_url,
    })
}

fn get_index_name(body: &str) -> Result<String> {
    Ok(Regex::new(r#"(?<=vm\.IndexName = ").*(?=";)"#)?
        .find(body)?
        .ok_or_else(|| anyhow!("regext not found anything"))?
        .as_str()
        .to_string())
}

fn get_vm_dir(body: &str) -> Result<String> {
    Ok(Regex::new(r#"(?<=vm\.Chapters = )\[.*\](?=;)"#)?
        .find(body)?
        .ok_or_else(|| anyhow!("regext not found anything"))?
        .as_str()
        .to_string())
}

fn get_ch_dirs(vm_dir: &str, index_name: &str) -> Result<Vec<DirChapter>> {
    Ok(serde_json::from_str::<Vec<DirChapter>>(vm_dir)?
        .iter()
        .map(|d| DirChapter {
            index_name: index_name.to_string(),
            ..d.clone()
        })
        .collect())
}

pub fn get_chapters(source_id: i64, url: &str, path: String) -> Result<Vec<ChapterInfo>> {
    let body = ureq::get(&format!("{}{}", url, path))
        .call()?
        .into_string()?;
    let index_name = get_index_name(&body)?;
    let vm_dir = get_vm_dir(&body)?;
    let ch_dirs = get_ch_dirs(&vm_dir, &index_name)?;

    let mut chapters = vec![];
    for ch in ch_dirs.iter() {
        let mut chapter = ch.chapter.clone();
        let t = chapter.remove(0);

        let index = if t != '1' {
            format!("-index-{}", t)
        } else {
            "".to_string()
        };

        chapter.insert(chapter.len() - 1, '.');
        let number = chapter.parse::<f64>().unwrap_or_default();

        chapters.push(ChapterInfo {
            source_id,
            title: format!("{} {}", ch.type_field, number.to_string()),
            path: format!(
                "/read-online/{}-chapter-{}{}.html",
                &index_name,
                number.to_string(),
                index,
            ),
            uploaded: ch.date.timestamp(),
            number: number + if index.is_empty() { 0.0 } else { 10000.0 },
            scanlator: None,
        })
    }

    Ok(chapters)
}

pub fn get_pages(url: &str, path: String) -> Result<Vec<String>> {
    let body = ureq::get(&format!("{}{}", url, path))
        .call()?
        .into_string()?;
    let index_name = get_index_name(&body)?;
    let cur_chapter = {
        let mat = Regex::new(r"(?<=vm\.CurChapter = ){.*}(?=;)")?
            .find(&body)?
            .ok_or_else(|| anyhow!("regext not found anything"))?
            .as_str()
            .to_string();
        serde_json::from_str::<CurChapter>(&mat)?
    };
    let cur_path_name = Regex::new(r#"(?<=vm\.CurPathName = ").*(?=";)"#)?
        .find(&body)?
        .ok_or_else(|| anyhow!("regext not found anything"))?
        .as_str()
        .to_string();
    let directory = {
        if cur_chapter.directory.is_empty() {
            "".to_string()
        } else {
            format!("{}/", cur_chapter.directory)
        }
    };
    let chapter_image = {
        let chapter = cur_chapter.chapter[1..cur_chapter.chapter.len() - 1].to_string();
        let odd = cur_chapter.chapter[cur_chapter.chapter.len() - 1..].to_string();
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
            let s = format!("000{}", i);
            s[(s.len() - 3)..].to_string()
        };

        pages.push(format!(
            "https://{}/manga/{}/{}{}-{}.png",
            cur_path_name, index_name, directory, chapter_image, page_image
        ));
    }

    Ok(pages)
}
