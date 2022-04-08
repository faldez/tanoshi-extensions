use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use fancy_regex::Regex;
use scraper::{ElementRef, Html, Selector};
use tanoshi_lib::prelude::{ChapterInfo, MangaInfo};

fn get_title(el: &ElementRef, selector: &str, attr: &str) -> Option<String> {
    let selector = Selector::parse(selector).ok()?;
    el.select(&selector)
        .next()?
        .value()
        .attr(attr)
        .map(|src| src.to_owned())
}

fn get_path(el: &ElementRef, selector: &str) -> Option<String> {
    let selector = Selector::parse(selector).ok()?;
    el.select(&selector)
        .next()?
        .value()
        .attr("href")
        .and_then(|href| url::Url::parse(href).ok())
        .map(|href| href.path().to_owned())
}

fn get_cover(el: &ElementRef, selector: &str) -> Option<String> {
    let selector = Selector::parse(selector).ok()?;
    el.select(&selector)
        .next()?
        .value()
        .attr("src")
        .map(|src| src.to_owned())
}

pub fn parse_manga_list(source_id: i64, body: &str, selector: &str) -> Result<Vec<MangaInfo>> {
    let doc = Html::parse_document(body);
    let selector =
        Selector::parse(selector).map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let mut manga = vec![];
    for el in doc.select(&selector) {
        manga.push(MangaInfo {
            source_id,
            title: get_title(&el, "a", "title").unwrap_or_default(),
            author: vec![],
            genre: vec![],
            status: None,
            description: None,
            path: get_path(&el, "a").unwrap_or_default(),
            cover_url: get_cover(&el, "a > img").unwrap_or_default(),
        })
    }

    Ok(manga)
}

pub fn parse_search_manga_list(
    source_id: i64,
    body: &str,
    selector: &str,
) -> Result<Vec<MangaInfo>> {
    let doc = Html::parse_document(body);
    let selector =
        Selector::parse(selector).map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let mut manga = vec![];
    for el in doc.select(&selector) {
        manga.push(MangaInfo {
            source_id,
            title: get_title(&el, "a > img", "alt").unwrap_or_default(),
            author: vec![],
            genre: vec![],
            status: None,
            description: None,
            path: get_path(&el, "a").unwrap_or_default(),
            cover_url: get_cover(&el, "a > img").unwrap_or_default(),
        })
    }

    Ok(manga)
}

pub fn get_manga_detail(path: &str, source_id: i64) -> Result<MangaInfo> {
    let body = ureq::get(&format!("https://readmanganato.com{path}"))
        .call()?
        .into_string()?;

    let doc = Html::parse_document(&body);

    let selector_img = Selector::parse(r#"span.info-image > img.img-loading"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_artist =
        Selector::parse(r#".story-info-right a[href^="https://manganato.com/author"]"#)
            .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_genre =
        Selector::parse(r#".story-info-right a[href^="https://manganato.com/genre"]"#)
            .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_desc = Selector::parse(r#"div#panel-story-info-description"#)
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
        description: doc.select(&selector_desc).next().map(|el| {
            let text = el.inner_html().trim().to_owned();
            // .replace(r#"<div id="panel-story-info-description" class="panel-story-info-description">"#, "")
            // .replace(r#"</div>"#, "")
            // .replace(r#"<h3>Description :</h3>"#, "");
            html2text::from_read(text.as_bytes(), 1000).replace("### Description :\n\n", "")
        }),
        path: path.to_string(),
        cover_url: doc
            .select(&selector_img)
            .find_map(|el| el.value().attr("src"))
            .map(|s| s.to_string())
            .unwrap_or_default(),
    })
}

pub fn get_chapters(path: &str, source_id: i64) -> Result<Vec<ChapterInfo>> {
    let body = ureq::get(&format!("https://readmanganato.com{path}"))
        .call()?
        .into_string()?;

    let doc = Html::parse_document(&body);

    let selector = Selector::parse(r#"ul.row-content-chapter > li"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_chapter_name = Selector::parse(r#"a.chapter-name"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_chapter_time = Selector::parse(r#"span.chapter-time"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let chapter_re = Regex::new(r#".*Chapter (\d+\.*\d*).*"#)?;

    let mut chapters = vec![];

    for el in doc.select(&selector) {
        let chapter_name = el
            .select(&selector_chapter_name)
            .flat_map(|el| el.text())
            .collect::<Vec<&str>>()
            .join("");
        let chapter_time = el
            .select(&selector_chapter_time)
            .next()
            .and_then(|el| el.value().attr("title"))
            .map(|title| title.to_string())
            .unwrap_or_else(|| "".to_string());
        chapters.push(ChapterInfo {
            source_id,
            title: chapter_name.clone(),
            path: el
                .select(&selector_chapter_name)
                .map(|el| {
                    el.value()
                        .attr("href")
                        .and_then(|href| url::Url::parse(href).ok())
                        .map(|href| href.path().to_owned())
                })
                .flatten()
                .collect::<Vec<String>>()
                .join(""),
            number: chapter_re
                .captures(&chapter_name)?
                .and_then(|cap| cap.get(0))
                .and_then(|num| num.as_str().parse().ok())
                .unwrap_or_default(),
            scanlator: None,
            uploaded: NaiveDateTime::parse_from_str(&chapter_time, "%b %d,%Y %H:%M")
                .unwrap_or_else(|_| NaiveDateTime::from_timestamp(0, 0))
                .timestamp(),
        });
    }

    Ok(chapters)
}

pub fn get_pages(body: &str) -> Result<Vec<String>> {
    let doc = Html::parse_document(&body);

    let selector = Selector::parse(r#".container-chapter-reader > img"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    Ok(doc
        .select(&selector)
        .flat_map(|el| el.value().attr("src"))
        .map(|p| p.to_string())
        .collect())
}
