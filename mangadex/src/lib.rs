pub mod mangadex;
use mangadex::Mangadex;
use tanoshi_lib::extensions::PluginRegistrar;

tanoshi_lib::export_plugin!(register);

extern "C" fn register(registrar: &mut dyn PluginRegistrar, _config: Option<&serde_yaml::Value>) {
    registrar.register_function("mangadex", Box::new(Mangadex{ /* fields */ }));
}
