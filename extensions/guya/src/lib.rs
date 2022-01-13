use guyalib::{get_chapters, get_manga_detail, get_manga_list, get_pages};
use tanoshi_lib::prelude::{Extension, Lang, PluginRegistrar};

const ID: i64 = 7;
const NAME: &str = "guya";
const URL: &str = "https://guya.moe";

tanoshi_lib::export_plugin!(register);

fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_function(Box::new(Guya::default()));
}

#[derive(Default)]
pub struct Guya;

impl Extension for Guya {
    fn get_source_info(&self) -> tanoshi_lib::prelude::SourceInfo {
        tanoshi_lib::prelude::SourceInfo {
            id: ID,
            name: NAME.to_string(),
            url: URL.to_string(),
            version: env!("CARGO_PKG_VERSION"),
            icon: "https://guya.moe/static/logo_small.png",
            languages: Lang::Single("en".to_string()),
            nsfw: false,
        }
    }

    fn get_popular_manga(
        &self,
        _page: i64,
    ) -> anyhow::Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        get_manga_list(URL, ID)
    }

    fn get_latest_manga(&self, _page: i64) -> anyhow::Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        get_manga_list(URL, ID)
    }

    fn search_manga(
        &self,
        _page: i64,
        query: Option<String>,
        _filters: Option<Vec<tanoshi_lib::prelude::Input>>,
    ) -> anyhow::Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        let manga = get_manga_list(URL, ID)?;

        if let Some(query) = query {
            Ok(manga
                .into_iter()
                .filter(|m| m.title.to_lowercase().contains(&query.to_lowercase()))
                .collect())
        } else {
            Ok(manga)
        }
    }

    fn get_manga_detail(&self, path: String) -> anyhow::Result<tanoshi_lib::prelude::MangaInfo> {
        get_manga_detail(URL, &path, ID)
    }

    fn get_chapters(&self, path: String) -> anyhow::Result<Vec<tanoshi_lib::prelude::ChapterInfo>> {
        get_chapters(URL, &path, ID)
    }

    fn get_pages(&self, path: String) -> anyhow::Result<Vec<String>> {
        get_pages(URL, &path)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_popular_manga() {
        let guya = Guya::default();
        let res = guya.get_popular_manga(1).unwrap();
        assert!(!res.is_empty());
    }

    #[test]
    fn test_get_latest_manga() {
        let guya = Guya::default();
        let res = guya.get_latest_manga(1).unwrap();
        assert!(!res.is_empty());
    }

    #[test]
    fn test_search_manga() {
        let guya = Guya::default();
        let res = guya
            .search_manga(1, Some("kaguya".to_string()), None)
            .unwrap();
        assert!(!res.is_empty());
    }

    #[test]
    fn test_get_manga_detail() {
        let guya = Guya::default();
        let res = guya
            .get_manga_detail("/api/series/Kaguya-Wants-To-Be-Confessed-To/".to_string())
            .unwrap();
        assert_eq!(res.title, "Kaguya-sama: Love is War");
    }

    #[test]
    fn test_get_chapters() {
        let guya = Guya::default();
        let res = guya
            .get_chapters("/api/series/Kaguya-Wants-To-Be-Confessed-To/".to_string())
            .unwrap();
        assert!(!res.is_empty());
    }

    #[test]
    fn test_get_pages() {
        let guya = Guya::default();
        let res = guya
            .get_pages("/api/series/Kaguya-Wants-To-Be-Confessed-To/1".to_string())
            .unwrap();
        assert!(!res.is_empty());
    }
}
