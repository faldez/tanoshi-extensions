use anyhow::{anyhow, Result};
use fancy_regex::Regex;
use std::io::BufReader;
use std::{fs, io};
use tanoshi_lib::extensions::Extension;
use tanoshi_lib::manga::{Chapter, Manga, Params, Source};

pub static NAME: &str = "local";

#[derive(Default)]
pub struct Local {
    pub url: String,
}

impl Local {
    pub fn new(path: &String) -> Local {
        Local {
            url: path.to_string(),
        }
    }
}

impl Extension for Local {
    fn info(&self) -> Source {
        Source {
            name: NAME.to_string(),
            url: self.url.clone(),
            version: std::env!("PLUGIN_VERSION").to_string(),
        }
    }

    fn get_mangas(&self, _param: Params, _auth: String) -> Result<Vec<Manga>> {
        let local_path = self.url.clone();
        let entries = fs::read_dir(&self.url)
            .expect("error read directory")
            .filter(|res| res.as_ref().unwrap().file_type().unwrap().is_dir())
            .map(|res| {
                res.map(|e| Manga {
                    id: 0,
                    source: NAME.to_string(),
                    title: e.file_name().to_str().unwrap().to_string(),
                    author: vec![],
                    genre: vec![],
                    status: None,
                    description: None,
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

    fn get_manga_info(&self, _path: &String) -> Result<Manga> {
        Ok(Manga::default())
    }

    fn get_chapters(&self, path: &String) -> Result<Vec<Chapter>> {
        let vol_re = Regex::new(r"(?i)(?<=v)(\d+)|(?<=volume)\s*(\d+)|(?<=vol)\s*(\d+)").unwrap();
        let ch_re = Regex::new(r"(?i)(?<=ch)(\d+)|(?<=chapter)\s*(\d+)").unwrap();

        let url = format!("{}{}", &self.url, &path);
        let local_path = self.url.clone();
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
                    let mat = vol_re.find(file_name.as_str()).unwrap();
                    ch.vol = mat.map(|m| m.as_str().to_string());
                    let mat = ch_re.find(file_name.as_str()).unwrap();
                    ch.no = mat.map(|m| m.as_str().to_string());
                    ch.title = Some(file_name);
                    ch.path = e
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

    fn get_pages(&self, path: &String) -> Result<Vec<String>> {
        let url = format!("{}{}", &self.url, &path);
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

    fn get_page(&self, path: &String) -> Result<Vec<u8>> {
        let url = format!("{}{}", &self.url, &path);
        let path = std::path::Path::new(&url);
        let dir = path.parent().unwrap().to_str().unwrap();
        let file_name = path.file_name().unwrap().to_str().unwrap();

        let file = fs::File::open(&dir)?;
        let reader = BufReader::new(file);

        let mut archive = zip::ZipArchive::new(reader)?;
        let mut zip_file = archive.by_name(file_name)?;
        let mut bytes = vec![];
        if io::copy(&mut zip_file, &mut bytes).is_err() {
            return Err(anyhow!("error write image"));
        }

        Ok(bytes)
    }
}
