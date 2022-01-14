mod dto;
use std::collections::HashMap;

use anyhow::Result;
use tanoshi_lib::prelude::*;

use crate::dto::{Detail, Series};

pub fn get_manga_list(url: &str, source_id: i64) -> Result<Vec<MangaInfo>> {
    let results: HashMap<String, Detail> = ureq::get(&format!("{}/api/get_all_series", url))
        .call()?
        .into_json()?;

    let mut manga: Vec<MangaInfo> = results
        .into_iter()
        .map(|(title, detail)| MangaInfo {
            source_id,
            title,
            author: vec![detail.author, detail.artist],
            genre: vec![],
            status: Some("Ongoing".to_string()),
            description: Some(detail.description),
            path: format!("/api/series/{}", detail.slug),
            cover_url: format!("{}{}", url, detail.cover),
        })
        .collect();

    manga.sort_by(|a, b| a.title.cmp(&b.title));

    Ok(manga)
}

pub fn get_manga_detail(url: &str, path: &str, source_id: i64) -> Result<MangaInfo> {
    let series: Series = ureq::get(&format!("{}{}", url, path)).call()?.into_json()?;

    Ok(MangaInfo {
        source_id,
        title: series.title.clone(),
        author: vec![series.author.clone(), series.author.clone()],
        genre: vec![],
        status: Some("Ongoing".to_string()),
        description: Some(series.description.clone()),
        path: path.to_string(),
        cover_url: format!("{}{}", url, series.cover),
    })
}

pub fn get_chapters(url: &str, path: &str, source_id: i64) -> Result<Vec<ChapterInfo>> {
    let series: Series = ureq::get(&format!("{}{}", url, path)).call()?.into_json()?;

    let mut chapters = vec![];

    for (number, chapter) in series.chapters {
        chapters.push(ChapterInfo {
            source_id,
            title: chapter.title.clone(),
            path: format!("{}{}", path, number),
            number: number.parse().unwrap_or_default(),
            scanlator: if let Some(group) = chapter.groups.into_keys().next() {
                series.groups.clone().get(&group).cloned()
            } else {
                None
            },
            uploaded: if let Some(date) = chapter.release_date.into_values().next() {
                date as i64
            } else {
                0
            },
        })
    }

    Ok(chapters)
}

pub fn get_pages(url: &str, path: &str) -> Result<Vec<String>> {
    let split: Vec<_> = path.rsplitn(2, '/').collect();
    let series: Series = ureq::get(&format!("{}{}", url, split[1]))
        .call()?
        .into_json()?;

    let pages = series
        .chapters
        .get(split[0])
        .and_then(|chapter| {
            chapter
                .groups
                .iter()
                .next()
                .map(|(group, pages)| (chapter.folder.clone(), group, pages))
        })
        .map(|(folder, group, pages)| {
            pages
                .iter()
                .map(|page| {
                    format!(
                        "{}/media/manga/{}/chapters/{}/{}/{}",
                        url, series.slug, folder, group, page
                    )
                })
                .collect()
        })
        .unwrap_or(vec![]);

    Ok(pages)
}
