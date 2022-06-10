use anyhow::bail;
use tanoshi_lib::prelude::{Extension, Lang, PluginRegistrar, SourceInfo};
use wpmangareader::{
    get_chapters, get_latest_manga, get_manga_detail, get_pages, get_popular_manga, search_manga,
};

tanoshi_lib::export_plugin!(register);

fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_function(Box::new(AlphaScans::default()));
}

const ID: i64 = 21;
const NAME: &str = "Alpha Scans";
const URL: &str = "https://alpha-scans.org";

#[derive(Default)]
pub struct AlphaScans;

impl Extension for AlphaScans {
    fn get_source_info(&self) -> SourceInfo {
        SourceInfo {
            id: ID,
            name: NAME.to_string(),
            url: URL.to_string(),
            version: env!("CARGO_PKG_VERSION"),
            icon: "https://i2.wp.com/alpha-scans.org/wp-content/uploads/2022/02/cropped-Alpha-logo-192x192.png",
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
        let AlphaScans = AlphaScans::default();

        let res1 = AlphaScans.get_latest_manga(1).unwrap();
        assert!(!res1.is_empty());

        let res2 = AlphaScans.get_latest_manga(2).unwrap();
        assert!(!res2.is_empty());

        assert_ne!(
            res1[0].path, res2[0].path,
            "{} should be different than {}",
            res1[0].path, res2[0].path
        );
    }

    #[test]
    fn test_get_popular_manga() {
        let AlphaScans = AlphaScans::default();

        let res = AlphaScans.get_popular_manga(1).unwrap();
        assert!(!res.is_empty());
    }

    #[test]
    fn test_search_manga() {
        let AlphaScans = AlphaScans::default();

        let res = AlphaScans
            .search_manga(1, Some("Star".to_string()), None)
            .unwrap();

        assert!(!res.is_empty());
    }

    #[test]
    fn test_get_manga_detail() {
        let AlphaScans = AlphaScans::default();

        let res = AlphaScans
            .get_manga_detail("/manga/star-instructor-master-baek/".to_string())
            .unwrap();

        assert_eq!(res.title, "Star Instructor, Master Baek");
    }

    #[test]
    fn test_get_chapters() {
        let AlphaScans = AlphaScans::default();

        let res = AlphaScans
            .get_chapters("/manga/star-instructor-master-baek/".to_string())
            .unwrap();

        assert!(!res.is_empty());
        println!("{res:?}");
    }

    #[test]
    fn test_get_pages() {
        let AlphaScans = AlphaScans::default();

        let res = AlphaScans
            .get_pages("/star-instructormaster-baek-chapter-1/".to_string())
            .unwrap();

        assert!(!res.is_empty());
    }
}
