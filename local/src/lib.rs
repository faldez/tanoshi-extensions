pub mod local;
use local::Local;
use tanoshi_lib::extensions::PluginRegistrar;

tanoshi_lib::export_plugin!(register);

extern "C" fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_function("local", Box::new(Local{ /* fields */ }));
}
