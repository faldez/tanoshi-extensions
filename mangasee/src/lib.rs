pub mod mangasee;
use mangasee::Mangasee;
use tanoshi_lib::extensions::PluginRegistrar;

tanoshi_lib::export_plugin!(mangasee::NAME, register);

fn register(registrar: &mut dyn PluginRegistrar, _config: Option<&serde_yaml::Value>) {
    registrar.register_function("mangasee", Box::new(Mangasee::new()));
}
