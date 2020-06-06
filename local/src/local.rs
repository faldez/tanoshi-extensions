use anyhow::{anyhow, Result};
use fancy_regex::Regex;
use std::io::BufReader;
use std::{fs, io};
use tanoshi_lib::extensions::Extension;
use tanoshi_lib::manga::{Chapter, Manga, Params, Source};

pub struct Local {}

impl Extension for Local {
    fn info(&mut self) -> Source {
        Source {
            id: 0,
            name: "local".to_string(),
            url: std::env::var("MANGA_PATH").unwrap_or("./manga".to_string()),
            version: std::env!("PLUGIN_VERSION").to_string(),
        }
    }

    fn get_mangas(&mut self, url: &String, _param: Params, _auth: String) -> Result<Vec<Manga>> {
        let local_path = std::env::var("MANGA_PATH").expect("MANGA_PATH not set");
        let entries = fs::read_dir(url.clone())
            .expect("error read directory")
            .filter(|res| res.as_ref().unwrap().file_type().unwrap().is_dir())
            .map(|res| {
                res.map(|e| Manga {
                    id: 0,
                    title: e.file_name().to_str().unwrap().to_string(),
                    author: "".to_string(),
                    //genre: vec![],
                    status: "".to_string(),
                    description: "".to_string(),
                    path: e
                        .path()
                        .to_str()
                        .unwrap()
                        .replace(local_path.as_str(), "")
                        .to_string(),
                    thumbnail_url: "".to_string(),
                    last_read: None,
                    last_page: None,
                    is_favorite: false,
                })
            })
            .collect::<Result<Vec<_>, io::Error>>()
            .unwrap_or(vec![]);

        Ok(entries)
    }

    fn get_manga_info(&mut self, _url: &String) -> Result<Manga> {
        Ok(Manga::default())
    }

    fn get_chapters(&mut self, url: &String) -> Result<Vec<Chapter>> {
        let local_path = std::env::var("MANGA_PATH").expect("MANGA_PATH not set");
        let re = Regex::new(r"(?<=v)(\d+)|(?<=volume)\s*(\d+)|(?<=vol)\s*(\d+)|(?<=ch)(\d+)|(?<=chapter)\s*(\d+)|(\d+)").unwrap();
        let entries = fs::read_dir(url)?
            .filter(|res| {
                res.as_ref().unwrap().file_type().unwrap().is_file()
                    && !res
                        .as_ref()
                        .unwrap()
                        .file_name()
                        .as_os_str()
                        .to_str()
                        .unwrap()
                        .starts_with(".")
            })
            .map(|res| {
                res.map(|e| {
                    let mut ch = Chapter::default();
                    let file_name = e.file_name().to_str().unwrap().to_string();
                    let mat = re.find(file_name.as_str()).unwrap();
                    ch.no = mat.unwrap().as_str().to_string();
                    ch.title = file_name;
                    ch.url = e
                        .path()
                        .to_str()
                        .unwrap()
                        .replace(local_path.as_str(), "")
                        .to_string();
                    ch
                })
            })
            .collect::<Result<Vec<_>, io::Error>>()
            .unwrap_or(vec![]);

        Ok(entries)
    }

    fn get_pages(&mut self, url: &String) -> Result<Vec<String>> {
        let file = fs::File::open(&url).unwrap();
        let reader = BufReader::new(file);

        let archive = zip::ZipArchive::new(reader).unwrap();
        let mut pages: Vec<String> = archive
            .file_names()
            .map(|file_name| format!("{}/{}", url, file_name))
            .collect();
        pages.sort();
        Ok(pages)
    }

    fn get_page(&mut self, url: &String, bytes: &mut Vec<u8>) -> Result<String> {
        let path = std::path::Path::new(url);
        let dir = path.parent().unwrap().to_str().unwrap();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let file_ext = path.extension().unwrap().to_str().unwrap();

        let file = fs::File::open(&dir)?;
        let reader = BufReader::new(file);

        let mut archive = zip::ZipArchive::new(reader)?;
        let mut zip_file = archive.by_name(file_name)?;
        if io::copy(&mut zip_file, bytes).is_err() {
            return Err(anyhow!("error write image"));
        }

        Ok(format!("image/{}", file_ext.to_string()))
    }
}
