use anyhow::bail;
use mangakakalot_common::{
    get_chapters, get_manga_detail, get_pages, parse_manga_list, parse_search_manga_list,
};
use tanoshi_lib::prelude::{Extension, Lang, PluginRegistrar, SourceInfo};

tanoshi_lib::export_plugin!(register);

fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_function(Box::new(Manganato::default()));
}

const ID: i64 = 11;
const NAME: &str = "Manganato";
const URL: &str = "https://manganato.com";

#[derive(Default)]
pub struct Manganato;

impl Extension for Manganato {
    fn get_source_info(&self) -> tanoshi_lib::prelude::SourceInfo {
        SourceInfo {
            id: ID,
            name: NAME.to_string(),
            url: URL.to_string(),
            version: env!("CARGO_PKG_VERSION"),
            icon: "https://manganato.com/favicon.ico",
            languages: Lang::Single("en".to_string()),
            nsfw: false,
        }
    }

    fn get_popular_manga(&self, page: i64) -> anyhow::Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        let body = ureq::get(&format!("{URL}/genre-all/{page}?type=topview"))
            .call()?
            .into_string()?;
        parse_manga_list(ID, &body, ".content-genres-item")
    }

    fn get_latest_manga(&self, page: i64) -> anyhow::Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        let body = ureq::get(&format!("{URL}/genre-all/{page}",))
            .call()?
            .into_string()?;
        parse_manga_list(ID, &body, ".content-genres-item")
    }

    fn search_manga(
        &self,
        page: i64,
        query: Option<String>,
        _: Option<Vec<tanoshi_lib::prelude::Input>>,
    ) -> anyhow::Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        if let Some(query) = query {
            let body = ureq::get(&format!(
                "{URL}/search/story/{}?page={page}",
                query.replace(" ", "_").to_lowercase()
            ))
            .call()?
            .into_string()?;
            parse_search_manga_list(ID, &body, "div.search-story-item")
        } else {
            bail!("query can not be empty")
        }
    }

    fn get_manga_detail(&self, path: String) -> anyhow::Result<tanoshi_lib::prelude::MangaInfo> {
        get_manga_detail(&path, ID)
    }

    fn get_chapters(&self, path: String) -> anyhow::Result<Vec<tanoshi_lib::prelude::ChapterInfo>> {
        get_chapters(&path, ID)
    }

    fn get_pages(&self, path: String) -> anyhow::Result<Vec<String>> {
        let body = ureq::get(&format!("https://readmanganato.com{path}"))
            .call()?
            .into_string()?;
        get_pages(&body)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_latest_manga() {
        let manganato = Manganato::default();

        let res1 = manganato.get_latest_manga(1).unwrap();
        assert!(!res1.is_empty());

        let res2 = manganato.get_latest_manga(2).unwrap();
        assert!(!res2.is_empty());

        assert!(
            res1[0].path != res2[0].path,
            "{} should be different than {}",
            res1[0].path,
            res2[0].path
        );
    }

    #[test]
    fn test_get_popular_manga() {
        let manganato = Manganato::default();

        let res = manganato.get_popular_manga(1).unwrap();
        assert!(!res.is_empty());
    }

    #[test]
    fn test_search_manga() {
        let manganato = Manganato::default();

        let res = manganato
            .search_manga(1, Some("one piece".to_string()), None)
            .unwrap();
        assert!(!res.is_empty());
    }

    #[test]
    fn test_get_manga_detail() {
        let manganato = Manganato::default();

        let res = manganato
            .get_manga_detail("/manga-hs951953".to_string())
            .unwrap();
        assert_eq!(res.title, "Shokugeki no Soma");
        assert_eq!(
            res.cover_url,
            "https://avt.mkklcdnv6temp.com/22/k/1-1583464578.jpg"
        );
        assert!(res.description.is_some());
        assert_ne!(res.description, Some("".to_string()));
        assert_eq!(
            res.author,
            vec!["Tsukuda Yuuto".to_string(), "Saeki Shun".to_string()]
        );
        assert_eq!(
            res.genre,
            vec![
                "Comedy".to_string(),
                "Cooking".to_string(),
                "Drama".to_string(),
                "Ecchi".to_string(),
                "School life".to_string(),
                "Shounen".to_string(),
            ]
        );
    }

    #[test]
    fn test_get_chapters() {
        let manganato = Manganato::default();

        let res = manganato
            .get_chapters("/manga-hs951953".to_string())
            .unwrap();
        assert!(!res.is_empty());
        println!("{res:?}");
    }

    #[test]
    fn test_get_pages() {
        let manganato = Manganato::default();

        let res = manganato
            .get_pages("/manga-hs951953/chapter-315.3".to_string())
            .unwrap();

        assert!(!res.is_empty());
    }
}
