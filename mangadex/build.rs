fn main() {
    println!(
        "cargo:rustc-env=PLUGIN_VERSION={}",
        env!("CARGO_PKG_VERSION")
    );
}
