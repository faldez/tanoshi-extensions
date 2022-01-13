use anyhow::Result;
use tanoshi_lib::prelude::{Extension, Lang, SourceInfo};

use tanoshi_lib::extensions::PluginRegistrar;

tanoshi_lib::export_plugin!(register);

fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_function(Box::new(Mangasee {}));
}

const ID: i64 = 3;
const NAME: &str = "MangaSee";
const URL: &str = "https://mangasee123.com";

pub struct Mangasee;

impl Extension for Mangasee {
    fn get_source_info(&self) -> tanoshi_lib::prelude::SourceInfo {
        SourceInfo {
            id: ID,
            name: NAME.to_string(),
            url: URL.to_string(),
            version: env!("CARGO_PKG_VERSION"),
            icon: "https://mangasee123.com/media/favicon.png",
            languages: Lang::Single("en".to_string()),
            nsfw: false,
        }
    }

    fn set_preferences(&mut self, _: Vec<tanoshi_lib::prelude::Input>) -> Result<()> {
        Ok(())
    }

    fn filter_list(&self) -> Vec<tanoshi_lib::prelude::Input> {
        nepnep::get_filter_list()
    }

    fn get_popular_manga(&self, page: i64) -> Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        nepnep::get_popular_manga(ID, URL, page)
    }

    fn get_latest_manga(&self, page: i64) -> Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        nepnep::get_latest_manga(ID, URL, page)
    }

    fn search_manga(
        &self,
        page: i64,
        query: Option<String>,
        filters: Option<Vec<tanoshi_lib::prelude::Input>>,
    ) -> Result<Vec<tanoshi_lib::prelude::MangaInfo>> {
        nepnep::search_manga(ID, URL, page, query, filters)
    }

    fn get_manga_detail(&self, path: String) -> Result<tanoshi_lib::prelude::MangaInfo> {
        nepnep::get_manga_detail(ID, URL, path)
    }

    fn get_chapters(&self, path: String) -> Result<Vec<tanoshi_lib::prelude::ChapterInfo>> {
        nepnep::get_chapters(ID, URL, path)
    }

    fn get_pages(&self, path: String) -> Result<Vec<String>> {
        nepnep::get_pages(URL, path)
    }
}
