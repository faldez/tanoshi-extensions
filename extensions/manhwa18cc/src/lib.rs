use anyhow::{anyhow, bail};
use madara::{get_chapters_old, get_manga_detail, parse_manga_list, search_manga_old};
use scraper::{Html, Selector};
use tanoshi_lib::prelude::{Extension, Lang, PluginRegistrar, SourceInfo};

tanoshi_lib::export_plugin!(register);

fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_function(Box::new(Manhwa18cc::default()));
}

const ID: i64 = 8;
const NAME: &str = "Manhwa18cc";
const URL: &str = "https://manhwa18.cc";

#[derive(Default)]
pub struct Manhwa18cc;

impl Extension for Manhwa18cc {
    fn get_source_info(&self) -> SourceInfo {
        SourceInfo {
            id: ID,
            name: NAME.to_string(),
            url: URL.to_string(),
            version: env!("CARGO_PKG_VERSION"),
            icon: "https://manhwa18.cc/images/favicon-160x160.png",
            languages: Lang::Multi(vec!["en".to_string(), "ko".to_string()]),
            nsfw: true,
        }
    }

    fn get_popular_manga(&self, page: i64) -> anyhow::Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        let body = ureq::get(&format!("{}/webtoons/{}?orderby=latest", URL, page))
            .call()?
            .into_string()?;

        let selector = Selector::parse(".manga-item")
            .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

        parse_manga_list(URL, ID, &body, &selector, false)
    }

    fn get_latest_manga(&self, page: i64) -> anyhow::Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        let body = ureq::get(&format!("{}/webtoons/{}?orderby=latest", URL, page))
            .call()?
            .into_string()?;

        let selector = Selector::parse(".manga-item")
            .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

        parse_manga_list(URL, ID, &body, &selector, false)
    }

    fn search_manga(
        &self,
        page: i64,
        query: Option<String>,
        _: Option<Vec<tanoshi_lib::prelude::Input>>,
    ) -> anyhow::Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        if let Some(query) = query {
            search_manga_old(URL, ID, page, &query)
        } else {
            bail!("query can not be empty")
        }
    }

    fn get_manga_detail(&self, path: String) -> anyhow::Result<tanoshi_lib::prelude::MangaInfo> {
        get_manga_detail(URL, &path, ID)
    }

    fn get_chapters(&self, path: String) -> anyhow::Result<Vec<tanoshi_lib::prelude::ChapterInfo>> {
        get_chapters_old(URL, &path, ID)
    }

    fn get_pages(&self, path: String) -> anyhow::Result<Vec<String>> {
        let body = ureq::get(&format!("{}{}", URL, path))
            .call()?
            .into_string()?;

        let doc = Html::parse_document(&body);

        let selector = Selector::parse(r#".read-content img"#)
            .map_err(|e| anyhow!("failed to parse selector: {:?}", e))?;

        Ok(doc
            .select(&selector)
            .flat_map(|el| {
                el.value()
                    .attr("data-src")
                    .or_else(|| el.value().attr("src"))
            })
            .map(|p| p.to_string())
            .collect())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_latest_manga() {
        let manhwa18cc = Manhwa18cc::default();

        let res1 = manhwa18cc.get_latest_manga(1).unwrap();
        assert!(!res1.is_empty());

        let res2 = manhwa18cc.get_latest_manga(2).unwrap();
        assert!(!res2.is_empty());

        assert_ne!(
            res1[0].path, res2[0].path,
            "{} should be different than {}",
            res1[0].path, res2[0].path
        );
    }

    #[test]
    fn test_get_popular_manga() {
        let manhwa18cc = Manhwa18cc::default();

        let res = manhwa18cc.get_popular_manga(1).unwrap();
        assert!(!res.is_empty());
    }

    #[test]
    fn test_search_manga() {
        let manhwa18cc = Manhwa18cc::default();

        let res = manhwa18cc
            .search_manga(1, Some("tutoring".to_string()), None)
            .unwrap();
        assert!(!res.is_empty());
    }

    #[test]
    fn test_get_manga_detail() {
        let manhwa18cc = Manhwa18cc::default();

        let res = manhwa18cc
            .get_manga_detail("/webtoon/private-tutoring-in-these-trying-times".to_string())
            .unwrap();
        assert_eq!(res.title, "Private Tutoring in These Trying Times");
    }

    #[test]
    fn test_get_chapters() {
        let manhwa18cc = Manhwa18cc::default();

        let res = manhwa18cc
            .get_chapters("/webtoon/private-tutoring-in-these-trying-times".to_string())
            .unwrap();
        assert!(!res.is_empty());
        println!("{res:?}");
    }

    #[test]
    fn test_get_pages() {
        let manhwa18cc = Manhwa18cc::default();

        let res = manhwa18cc
            .get_pages("//webtoon/private-tutoring-in-these-trying-times/chapter-27".to_string())
            .unwrap();

        assert!(!res.is_empty());
    }
}
