pub mod local;
use local::Local;
use tanoshi_lib::extensions::PluginRegistrar;

#[derive(Debug, serde::Deserialize, Default)]
pub struct Config {
    pub path: Option<String>,
}

tanoshi_lib::export_plugin!(register);

extern "C" fn register(registrar: &mut dyn PluginRegistrar, config: Option<&serde_yaml::Value>) {
    let config = config.unwrap_or(&serde_yaml::Value::default()).to_owned();
    let cfg: Config = serde_yaml::from_value(config).unwrap_or(Config::default());
    println!("{:?}", cfg);
    registrar.register_function(
        "local",
        Box::new(Local {
            url: cfg.path.unwrap_or("~/tanoshi/manga".to_string()),
        }),
    );
}
