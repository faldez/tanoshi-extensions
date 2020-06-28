extern crate libloading as lib;

mod extension;

use anyhow::Result;
use tanoshi_lib::extensions::Extension;

fn main() -> Result<()> {
    let mut exts = extension::Extensions::new();
    for entry in std::fs::read_dir("target/release")?
    .into_iter()
    .filter(move |path| {
        if let Ok(p) = path {
            let ext = p
                .clone()
                .path()
                .extension()
                .unwrap_or("".as_ref())
                .to_owned();
            if ext == "so" || ext == "dll" || ext == "dylib" {
                return true;
            }
        }
        return false;
    }) {
        let path = entry?.path();
        unsafe {
            exts.load(path, None)?;
        }
    }
    let mut sources = vec![];
    for (_, ext) in exts.extensions() {
        sources.push(ext.info());
    }

    let file = std::fs::File::create("index.json")?;
    serde_json::to_writer_pretty(&file, &sources)?;
    Ok(())
}
