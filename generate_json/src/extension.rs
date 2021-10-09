use anyhow::{anyhow, Result};
//use local::Local;
use lib::Library;
use std::{collections::HashMap, env, ffi::OsStr, fs, io, sync::Arc};
use tanoshi_lib::extensions::{Extension, PluginDeclaration};
use tanoshi_lib::manga::{Chapter, Image, Manga, Params, Source, SourceLogin, SourceLoginResult};

pub struct ExtensionProxy {
    extension: Box<dyn Extension>,
    _lib: Arc<Library>,
}

impl Extension for ExtensionProxy {
    fn info(&self) -> Source {
        self.extension.info()
    }

    fn get_mangas(&self, url: &String, param: Params, auth: String) -> Result<Vec<Manga>> {
        self.extension.get_mangas(url, param, auth)
    }

    fn get_manga_info(&self, url: &String) -> Result<Manga> {
        self.extension.get_manga_info(url)
    }

    fn get_chapters(&self, url: &String) -> Result<Vec<Chapter>> {
        self.extension.get_chapters(url)
    }

    fn get_pages(&self, url: &String) -> Result<Vec<String>> {
        self.extension.get_pages(url)
    }

    fn get_page(&self, image: Image) -> Result<Vec<u8>> {
        self.extension.get_page(image)
    }

    fn login(&self, login_info: SourceLogin) -> Result<SourceLoginResult> {
        self.extension.login(login_info)
    }
}

pub struct Extensions {
    extensions: HashMap<String, ExtensionProxy>,
    libraries: Vec<Arc<Library>>,
}

impl Extensions {
    pub fn new() -> Extensions {
        Extensions {
            extensions: HashMap::new(),
            libraries: vec![],
        }
    }

    pub fn extensions(&self) -> &HashMap<String, ExtensionProxy> {
        &self.extensions
    }

    pub fn get(&self, name: &String) -> Option<&ExtensionProxy> {
        self.extensions.get(name)
    }

    pub unsafe fn load<P: AsRef<OsStr>>(
        &mut self,
        library_path: P,
        config: Option<&serde_yaml::Value>,
    ) -> Result<()> {
        let library = Arc::new(Library::new(library_path)?);

        let decl = library
            .get::<*mut PluginDeclaration>(b"plugin_declaration\0")?
            .read();

        if decl.rustc_version != tanoshi_lib::RUSTC_VERSION
            || decl.core_version != tanoshi_lib::CORE_VERSION
        {
            return Err(anyhow!("Version mismatch"));
        }

        let mut registrar = PluginRegistrar::new(Arc::clone(&library));
        (decl.register)(&mut registrar, config);

        self.extensions.extend(registrar.extensions);
        self.libraries.push(library);

        Ok(())
    }
}

pub struct PluginRegistrar {
    extensions: HashMap<String, ExtensionProxy>,
    lib: Arc<Library>,
}

impl PluginRegistrar {
    fn new(lib: Arc<Library>) -> PluginRegistrar {
        PluginRegistrar {
            lib,
            extensions: HashMap::default(),
        }
    }
}

impl tanoshi_lib::extensions::PluginRegistrar for PluginRegistrar {
    fn register_function(&mut self, name: &str, extension: Box<dyn Extension>) {
        let proxy = ExtensionProxy {
            extension,
            _lib: Arc::clone(&self.lib),
        };

        self.extensions.insert(name.to_string(), proxy);
    }
}
