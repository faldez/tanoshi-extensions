use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use scraper::{ElementRef, Html, Selector};
use tanoshi_lib::prelude::{ChapterInfo, MangaInfo};

fn get_title(el: &ElementRef) -> Option<String> {
    el.parent()?
        .value()
        .as_element()?
        .attr("title")
        .map(|s| s.to_string())
}

fn get_href(el: &ElementRef) -> Option<String> {
    el.parent()?
        .value()
        .as_element()?
        .attr("href")
        .map(|s| s.to_string())
}

fn get_data_src(el: &ElementRef) -> Option<String> {
    el.value().attr("data-src").map(|s| s.to_string())
}

fn parse_manga_list(source_id: i64, body: &str) -> Result<Vec<MangaInfo>> {
    let doc = Html::parse_document(body);
    let selector = Selector::parse(r#".manga-item a[href^="/webtoon"] img"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let mut manga = vec![];
    for el in doc.select(&selector) {
        manga.push(MangaInfo {
            source_id,
            title: get_title(&el).unwrap_or_default(),
            author: vec![],
            genre: vec![],
            status: None,
            description: None,
            path: get_href(&el).unwrap_or_default(),
            cover_url: get_data_src(&el).unwrap_or_default(),
        })
    }

    Ok(manga)
}

pub fn get_latest_manga(url: &str, source_id: i64, page: i64) -> Result<Vec<MangaInfo>> {
    let body = ureq::get(&format!("{}/webtoons/{}orderby=latest", url, page))
        .call()?
        .into_string()?;
    parse_manga_list(source_id, &body)
}

pub fn get_popular_manga(url: &str, source_id: i64, page: i64) -> Result<Vec<MangaInfo>> {
    let body = ureq::get(&format!("{}/webtoons/{}orderby=trending", url, page))
        .call()?
        .into_string()?;
    parse_manga_list(source_id, &body)
}

pub fn search_manga(url: &str, source_id: i64, page: i64, query: &str) -> Result<Vec<MangaInfo>> {
    let body = ureq::get(&format!("{}/search?q={}&page={}", url, query, page))
        .call()?
        .into_string()?;
    parse_manga_list(source_id, &body)
}

pub fn get_manga_detail(url: &str, path: &str, source_id: i64) -> Result<MangaInfo> {
    let body = ureq::get(&format!("{}{}", url, path))
        .call()?
        .into_string()?;

    let doc = Html::parse_document(&body);

    let selector_img = Selector::parse(r#"a[href^="/webtoon"] img"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_artist = Selector::parse(r#".artist-content a"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_genre = Selector::parse(r#".genres-content a"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;
    let selector_desc = Selector::parse(r#".dsct > p"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    Ok(MangaInfo {
        source_id,
        title: doc
            .select(&selector_img)
            .find_map(|el| el.value().attr("title"))
            .map(|s| s.to_string())
            .unwrap_or_default(),
        author: doc
            .select(&selector_artist)
            .flat_map(|el| el.text())
            .map(|s| s.to_string())
            .collect(),
        genre: doc
            .select(&selector_genre)
            .flat_map(|el| el.text())
            .map(|s| s.to_string())
            .collect(),
        status: None,
        description: doc
            .select(&selector_desc)
            .flat_map(|el| el.text())
            .map(|s| s.to_string())
            .next(),
        path: path.to_string(),
        cover_url: doc
            .select(&selector_img)
            .find_map(|el| el.value().attr("data-src"))
            .map(|s| s.to_string())
            .unwrap_or_default(),
    })
}

pub fn get_chapters(url: &str, path: &str, source_id: i64) -> Result<Vec<ChapterInfo>> {
    let body = ureq::get(&format!("{}{}", url, path))
        .call()?
        .into_string()?;

    let doc = Html::parse_document(&body);

    let selector = Selector::parse(r#"#chapterlist .a-h.wleft"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_chapter_name = Selector::parse(r#".chapter-name"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_chapter_time = Selector::parse(r#".chapter-time"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let chapters = doc
        .select(&selector)
        .map(|el| {
            let chapter_name = el
                .select(&selector_chapter_name)
                .flat_map(|el| el.text())
                .collect::<Vec<&str>>()
                .join("");
            let chapter_time = el
                .select(&selector_chapter_time)
                .flat_map(|el| el.text())
                .collect::<Vec<&str>>()
                .join("");
            ChapterInfo {
                source_id,
                title: chapter_name.clone(),
                path: el
                    .select(&selector_chapter_name)
                    .map(|el| el.value().attr("href"))
                    .flatten()
                    .collect::<Vec<&str>>()
                    .join(""),
                number: chapter_name
                    .replace("CHapter ", "")
                    .parse()
                    .unwrap_or_default(),
                scanlator: None,
                uploaded: NaiveDateTime::parse_from_str(
                    &format!("{} 00:00:00", chapter_time),
                    "%d %b %Y %H:%M:%S",
                )
                .unwrap_or_else(|_| NaiveDateTime::from_timestamp(0, 0))
                .timestamp(),
            }
        })
        .collect();

    Ok(chapters)
}

pub fn get_pages(url: &str, path: &str) -> Result<Vec<String>> {
    let body = ureq::get(&format!("{}{}", url, path))
        .call()?
        .into_string()?;

    let doc = Html::parse_document(&body);

    let selector = Selector::parse(r#".read-content > img"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    Ok(doc
        .select(&selector)
        .flat_map(|el| el.value().attr("src"))
        .map(|p| p.to_string())
        .collect())
}
