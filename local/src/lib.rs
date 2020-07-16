pub mod local;
use dirs;
use local::Local;
use tanoshi_lib::extensions::PluginRegistrar;

#[derive(Debug, serde::Deserialize, Default)]
pub struct Config {
    pub path: Option<String>,
}

tanoshi_lib::export_plugin!(local::NAME, register);

extern "C" fn register(registrar: &mut dyn PluginRegistrar, config: Option<&serde_yaml::Value>) {
    let config = config.unwrap_or(&serde_yaml::Value::default()).to_owned();
    let cfg: Config = serde_yaml::from_value(config).unwrap_or(Config::default());

    registrar.register_function(
        "local",
        Box::new(Local::new(&cfg.path.unwrap_or_else(|| {
            dirs::home_dir()
                .expect("should have home dir")
                .join(".tanoshi")
                .join(".manga")
                .into_os_string()
                .into_string()
                .unwrap()
        }))),
    );
}
