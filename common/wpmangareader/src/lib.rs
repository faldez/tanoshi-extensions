use anyhow::{anyhow, Result};
use chrono::{NaiveDateTime, Utc};
use scraper::{ElementRef, Html, Selector};
use tanoshi_lib::prelude::{ChapterInfo, MangaInfo};

fn get_data_src(el: &ElementRef) -> Option<String> {
    el.value()
        .attr("data-lazy-src")
        .or_else(|| el.value().attr("data-src"))
        .or_else(|| el.value().attr("src"))
        .map(|s| s.to_string())
}

pub fn parse_manga_list(url: &str, source_id: i64, body: &str) -> Result<Vec<MangaInfo>> {
    let mut manga = vec![];

    let doc = Html::parse_document(body);

    let selector = Selector::parse(".utao .uta .imgu, .listupd .bs .bsx, .listo .bs .bsx")
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    for el in doc.select(&selector) {
        let selector_name =
            Selector::parse("a").map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

        let selector_img =
            Selector::parse("img").map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

        manga.push(MangaInfo {
            source_id,
            title: el
                .select(&selector_name)
                .next()
                .unwrap()
                .value()
                .attr("title")
                .unwrap()
                .trim()
                .to_string(),
            author: vec![],
            genre: vec![],
            status: None,
            description: None,
            path: el
                .select(&selector_name)
                .next()
                .unwrap()
                .value()
                .attr("href")
                .unwrap()
                .replace(url, ""),
            cover_url: get_data_src(&el.select(&selector_img).next().unwrap()).unwrap_or_default(),
        })
    }

    Ok(manga)
}

pub fn get_latest_manga(url: &str, source_id: i64, page: i64) -> Result<Vec<MangaInfo>> {
    let body = ureq::get(&format!("{}/manga/?page={}&order=latest", url, page))
        .set("Referer", url)
        .call()?
        .into_string()?;

    parse_manga_list(url, source_id, &body)
}

pub fn get_popular_manga(url: &str, source_id: i64, page: i64) -> Result<Vec<MangaInfo>> {
    let body = ureq::get(&format!("{}/manga/?page={}&order=popular", url, page))
        .set("Referer", url)
        .call()?
        .into_string()?;

    parse_manga_list(url, source_id, &body)
}

pub fn search_manga(url: &str, source_id: i64, page: i64, query: &str) -> Result<Vec<MangaInfo>> {
    let body = ureq::get(&format!("{}/page/{}/?s={}", url, page, query))
        .set("Referer", url)
        .call()?
        .into_string()?;

    parse_manga_list(url, source_id, &body)
}

pub fn get_manga_detail(url: &str, path: &str, source_id: i64) -> Result<MangaInfo> {
    let body = ureq::get(&format!("{}{}", url, path))
        .set("Referer", url)
        .call()?
        .into_string()?;

    let doc = Html::parse_document(&body);

    let selector_name = Selector::parse(r#"h1.entry-title"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_img = Selector::parse(".infomanga > div[itemprop=image] img, .thumb img")
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_genre = Selector::parse(r#"div.gnr a, .mgen a, .seriestugenre a"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_desc = Selector::parse(".desc, .entry-content[itemprop=description]")
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    Ok(MangaInfo {
        source_id,
        title: doc
            .select(&selector_name)
            .next()
            .and_then(|item| item.last_child())
            .and_then(|t| t.value().as_text())
            .unwrap()
            .trim()
            .to_string(),
        author: vec![],
        genre: doc
            .select(&selector_genre)
            .flat_map(|el| el.text())
            .map(|s| s.to_string())
            .collect(),
        status: None,
        description: Option::from(
            doc.select(&selector_desc)
                .flat_map(|el| el.text())
                .collect::<Vec<&str>>()
                .join("")
                .trim()
                .to_string(),
        ),
        path: path.to_string().replace(url, ""),
        cover_url: doc
            .select(&selector_img)
            .find_map(|el| get_data_src(&el))
            .unwrap_or_default(),
    })
}

pub fn get_chapters(url: &str, path: &str, source_id: i64) -> Result<Vec<ChapterInfo>> {
    let body = ureq::get(&format!("{}{}", url, path))
        .set("Referer", url)
        .set("X-Requested-With", "XMLHttpRequest")
        .call()?
        .into_string()?;

    let doc = Html::parse_document(&body);

    let selector = Selector::parse(r#"div.bxcl li, #chapterlist li .eph-num a"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_chapter_name = Selector::parse(r#".lch a, .chapternum"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_chapter_time = Selector::parse(r#".chapterdate"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_chapter_url =
        Selector::parse("a").map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let chapters: Vec<ChapterInfo> = doc
        .select(&selector)
        .map(|el| {
            let chapter_name = el
                .select(&selector_chapter_name)
                .flat_map(|el| el.text())
                .collect::<Vec<&str>>()
                .join("")
                .trim()
                .to_string();
            let chapter_time = el
                .select(&selector_chapter_time)
                .flat_map(|el| el.text())
                .collect::<Vec<&str>>()
                .join("");

            ChapterInfo {
                source_id,
                title: chapter_name.clone(),
                path: el
                    .select(&selector_chapter_url)
                    .filter_map(|el| el.value().attr("href"))
                    .collect::<Vec<&str>>()
                    .join("")
                    .replace(url, ""),
                number: chapter_name
                    .replace("Chapter ", "")
                    .split(' ')
                    .next()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or_default(),
                scanlator: None,
                uploaded: NaiveDateTime::parse_from_str(
                    &format!("{} 00:00", chapter_time.trim()),
                    "%B %d, %Y %H:%M",
                )
                .unwrap_or_else(|_| Utc::now().naive_utc())
                .timestamp(),
            }
        })
        .collect();

    Ok(chapters)
}

pub fn get_pages(url: &str, path: &str) -> Result<Vec<String>> {
    let body = ureq::get(&format!("{}{}", url, path))
        .set("Referer", url)
        .call()?
        .into_string()?;

    let doc = Html::parse_document(&body);

    let selector = Selector::parse(r#"div#readerarea img"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    Ok(doc
        .select(&selector)
        .flat_map(|el| get_data_src(&el))
        .map(|p| p.trim().to_string())
        .collect())
}
