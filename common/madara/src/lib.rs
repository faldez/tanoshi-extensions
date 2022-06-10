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

pub fn parse_manga_list(
    url: &str,
    source_id: i64,
    body: &str,
    selector: &Selector,
    is_selector_url: bool,
) -> Result<Vec<MangaInfo>> {
    let mut manga = vec![];

    let doc = Html::parse_document(body);

    for el in doc.select(selector) {
        let selector_name = Selector::parse(if is_selector_url {
            "div.item-summary > a > h3, div.data > h3 > a, div.post-title > h3"
        } else {
            "div.item-summary > a > h3, div.data > h3 > a, div.post-title > h3 > a"
        })
            .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

        let selector_url = Selector::parse("div.data a, div.post-title a, div.item-thumb a")
            .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

        let selector_img =
            Selector::parse("img").map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

        manga.push(MangaInfo {
            source_id,
            title: el
                .select(&selector_name)
                .next()
                .and_then(|item| item.last_child())
                .and_then(|t| t.value().as_text())
                .unwrap()
                .trim()
                .to_string(),
            author: vec![],
            genre: vec![],
            status: None,
            description: None,
            path: if is_selector_url {
                el.value().attr("href").unwrap().replace(url, "")
            } else {
                el.select(&selector_url)
                    .next()
                    .unwrap()
                    .value()
                    .attr("href")
                    .unwrap_or_default()
                    .replace(url, "")
            },
            cover_url: get_data_src(&el.select(&selector_img).next().unwrap()).unwrap_or_default(),
        })
    }

    Ok(manga)
}

pub fn get_latest_manga(url: &str, source_id: i64, page: i64) -> Result<Vec<MangaInfo>> {
    let body = ureq::post(&format!("{}/wp-admin/admin-ajax.php", url))
        .set("Referer", url)
        .send_form(&[
            ("action", "madara_load_more"),
            ("page", &(page - 1).to_string()),
            ("template", "madara-core/content/content-archive"),
            ("vars[orderby]", "meta_value_num"),
            ("vars[paged]", "1"),
            ("vars[posts_per_page]", "20"),
            ("vars[post_type]", "wp-manga"),
            ("vars[post_status]", "publish"),
            ("vars[meta_key]", "_latest_update"),
            ("vars[order]", "desc"),
            ("vars[sidebar]", "right"),
            ("vars[manga_archives_item_layout]", "big_thumbnail"),
            ("vars[meta_query][0][key]", "_wp_manga_chapter_type"),
            ("vars[meta_query][0][value]", "manga"),
        ])?
        .into_string()?;

    let selector = Selector::parse("div.page-item-detail")
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    parse_manga_list(url, source_id, &body, &selector, false)
}

pub fn get_popular_manga(url: &str, source_id: i64, page: i64) -> Result<Vec<MangaInfo>> {
    let body = ureq::post(&format!("{}/wp-admin/admin-ajax.php", url))
        .set("Referer", url)
        .send_form(&[
            ("action", "madara_load_more"),
            ("page", &(page - 1).to_string()),
            ("template", "madara-core/content/content-archive"),
            ("vars[orderby]", "meta_value_num"),
            ("vars[paged]", "1"),
            ("vars[posts_per_page]", "20"),
            ("vars[post_type]", "wp-manga"),
            ("vars[post_status]", "publish"),
            ("vars[meta_key]", "_wp_manga_views"),
            ("vars[order]", "desc"),
            ("vars[sidebar]", "full"),
            ("vars[manga_archives_item_layout]", "big_thumbnail"),
            ("vars[meta_query][0][key]", "_wp_manga_chapter_type"),
            ("vars[meta_query][0][value]", "manga"),
        ])?
        .into_string()?;

    let selector = Selector::parse("div.page-item-detail")
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    parse_manga_list(url, source_id, &body, &selector, false)
}

pub fn search_manga_old(
    url: &str,
    source_id: i64,
    page: i64,
    query: &str,
) -> Result<Vec<MangaInfo>> {
    let body = ureq::get(&format!("{}/search?q={}&page={}", url, query, page))
        .call()?
        .into_string()?;

    let selector =
        Selector::parse(".manga-item").map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    parse_manga_list(url, source_id, &body, &selector, false)
}

pub fn search_manga(
    url: &str,
    source_id: i64,
    page: i64,
    query: &str,
    is_selector_url: bool,
) -> Result<Vec<MangaInfo>> {
    let body = ureq::post(&format!("{}/wp-admin/admin-ajax.php", url))
        .set("Referer", url)
        .send_form(&[
            ("action", "madara_load_more"),
            ("vars[s]", query),
            ("template", "madara-core/content/content-search"),
            ("vars[paged]", "1"),
            ("vars[template]", "archive"),
            ("vars[post_type]", "wp-manga"),
            ("vars[post_status]", "publish"),
            ("vars[sidebar]", "right"),
            ("vars[manga_archives_item_layout]", "big_thumbnail"),
            ("vars[posts_per_page]", "20"),
            ("vars[meta_query][0][key]", "_wp_manga_chapter_type"),
            ("vars[meta_query][0][value]", "manga"),
            ("page", &(page - 1).to_string()),
        ])?
        .into_string()?;

    let selector = if is_selector_url {
        Selector::parse("a").map_err(|e| anyhow!("failed to parse selector: {:?}", e))?
    } else {
        Selector::parse("div.c-tabs-item__content")
            .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?
    };

    parse_manga_list(url, source_id, &body, &selector, is_selector_url)
}

pub fn get_manga_detail(url: &str, path: &str, source_id: i64) -> Result<MangaInfo> {
    let body = ureq::get(&format!("{}{}", url, path))
        .call()?
        .into_string()?;

    let doc = Html::parse_document(&body);

    let selector_name =
        Selector::parse(r#"div.post-title h3, div.post-title h1, div.series-title h1"#)
            .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_img = Selector::parse(".summary_image img, .series-img img")
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_artist = Selector::parse(".artist-content a")
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_genre = Selector::parse(r#".genres-content a"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_desc = Selector::parse("div.description-summary div.summary__content, div.summary_content div.post-content_item > h5 + div, div.summary_content div.manga-excerpt, div.summary-text p")
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

fn parse_chapters(
    url: &str,
    doc: &Html,
    selector: &Selector,
    selector_chapter_name: &Selector,
    selector_chapter_time: &Selector,
    selector_chapter_url: &Selector,
    source_id: i64,
) -> Result<Vec<ChapterInfo>> {
    let chapters: Vec<ChapterInfo> = doc
        .select(selector)
        .map(|el| {
            let chapter_name = el
                .select(selector_chapter_name)
                .flat_map(|el| el.text())
                .collect::<Vec<&str>>()
                .join("")
                .trim()
                .to_string();
            let chapter_time = el
                .select(selector_chapter_time)
                .flat_map(|el| el.text())
                .collect::<Vec<&str>>()
                .join("");

            ChapterInfo {
                source_id,
                title: chapter_name.clone(),
                path: el
                    .select(selector_chapter_url)
                    .next()
                    .unwrap()
                    .value()
                    .attr("href")
                    .unwrap()
                    .to_string()
                    .replace(url, ""),
                number: chapter_name
                    .replace("Chapter ", "")
                    .parse()
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

pub fn get_chapters_old(url: &str, path: &str, source_id: i64) -> Result<Vec<ChapterInfo>> {
    let body = ureq::get(&format!("{}{}", url, path))
        .set("Referer", url)
        .set("X-Requested-With", "XMLHttpRequest")
        .call()?
        .into_string()?;

    let doc = Html::parse_document(&body);

    let selector = Selector::parse(r#"#chapterlist .a-h.wleft"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_chapter_name = Selector::parse(r#".chapter-name"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_chapter_time = Selector::parse(r#".chapter-time"#)
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_chapter_url = Selector::parse(".chapter-name")
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    parse_chapters(
        url,
        &doc,
        &selector,
        &selector_chapter_name,
        &selector_chapter_time,
        &selector_chapter_url,
        source_id,
    )
}

pub fn get_chapters(
    url: &str,
    path: &str,
    source_id: i64,
    chapter_name_selector: Option<&str>,
) -> Result<Vec<ChapterInfo>> {
    let body = ureq::post(&format!("{}{}ajax/chapters", url, path))
        .set("Referer", url)
        .set("Content-Length", "0")
        .set("X-Requested-With", "XMLHttpRequest")
        .call()?
        .into_string()?;

    let doc = Html::parse_document(&body);

    let selector = Selector::parse("li.wp-manga-chapter,li.chapter-li")
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_chapter_name = Selector::parse(chapter_name_selector.unwrap_or("a"))
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_chapter_url =
        Selector::parse("a").map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    let selector_chapter_time = Selector::parse(".chapter-release-date")
        .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    parse_chapters(
        url,
        &doc,
        &selector,
        &selector_chapter_name,
        &selector_chapter_time,
        &selector_chapter_url,
        source_id,
    )
}

pub fn get_pages(url: &str, path: &str) -> Result<Vec<String>> {
    let body = ureq::get(&format!("{}{}", url, path))
        .set("Referer", url)
        .call()?
        .into_string()?;

    let doc = Html::parse_document(&body);

    let selector =
        Selector::parse(r#"div.page-break, li.blocks-gallery-item, .reading-content img"#)
            .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

    Ok(doc
        .select(&selector)
        .flat_map(|el| get_data_src(&el))
        .map(|p| p.trim().to_string())
        .collect())
}
