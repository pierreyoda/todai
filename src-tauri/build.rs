fn main() {
    // `generate_context!` embeds the icons (the Dock's in `tauri dev`) only when the crate recompiles: a regenerated
    // icon set must trigger it
    println!("cargo:rerun-if-changed=icons");
    tauri_build::build()
}
