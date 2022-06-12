use anyhow::bail;
use tanoshi_lib::prelude::{Extension, Lang, PluginRegistrar, SourceInfo};
use wpmangastream::{
    get_chapters, get_latest_manga, get_manga_detail, get_pages, get_popular_manga, search_manga,
};

tanoshi_lib::export_plugin!(register);

fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_function(Box::new(AsuraScans::default()));
}

const ID: i64 = 22;
const NAME: &str = "Asura Scans";
const URL: &str = "https://www.asurascans.com";

#[derive(Default)]
pub struct AsuraScans;

impl Extension for AsuraScans {
    fn get_source_info(&self) -> SourceInfo {
        SourceInfo {
            id: ID,
            name: NAME.to_string(),
            url: URL.to_string(),
            version: env!("CARGO_PKG_VERSION"),
            icon: "https://www.asurascans.com/wp-content/uploads/2021/03/cropped-Group_1-1-270x270.png",
            languages: Lang::Single("en".to_string()),
            nsfw: false,
        }
    }

    fn get_popular_manga(&self, page: i64) -> anyhow::Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        get_popular_manga(URL, ID, page)
    }

    fn get_latest_manga(&self, page: i64) -> anyhow::Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        get_latest_manga(URL, ID, page)
    }

    fn search_manga(
        &self,
        page: i64,
        query: Option<String>,
        _: Option<Vec<tanoshi_lib::prelude::Input>>,
    ) -> anyhow::Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        if let Some(query) = query {
            search_manga(URL, ID, page, &query)
        } else {
            bail!("query can not be empty")
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
    fn test_get_latest_manga() {
        let AsuraScans = AsuraScans::default();

        let res1 = AsuraScans.get_latest_manga(1).unwrap();
        assert!(!res1.is_empty());

        let res2 = AsuraScans.get_latest_manga(2).unwrap();
        assert!(!res2.is_empty());

        assert_ne!(
            res1[0].path, res2[0].path,
            "{} should be different than {}",
            res1[0].path, res2[0].path
        );
    }

    #[test]
    fn test_get_popular_manga() {
        let AsuraScans = AsuraScans::default();

        let res = AsuraScans.get_popular_manga(1).unwrap();
        assert!(!res.is_empty());
    }

    #[test]
    fn test_search_manga() {
        let AsuraScans = AsuraScans::default();

        let res = AsuraScans
            .search_manga(1, Some("Star".to_string()), None)
            .unwrap();

        assert!(!res.is_empty());
    }

    #[test]
    fn test_get_manga_detail() {
        let AsuraScans = AsuraScans::default();

        let res = AsuraScans
            .get_manga_detail("/comics/367-reincarnation-of-the-suicidal-battle-god/".to_string())
            .unwrap();

        assert_eq!(res.title, "Reincarnation of the Suicidal Battle God");
    }

    #[test]
    fn test_get_chapters() {
        let AsuraScans = AsuraScans::default();

        let res = AsuraScans
            .get_chapters("/comics/367-reincarnation-of-the-suicidal-battle-god/".to_string())
            .unwrap();

        assert!(!res.is_empty());
        println!("{res:?}");
    }

    #[test]
    fn test_get_pages() {
        let AsuraScans = AsuraScans::default();

        let res = AsuraScans
            .get_pages("/reincarnation-of-the-suicidal-battle-god-chapter-1/".to_string())
            .unwrap();

        assert!(!res.is_empty());
    }
}
