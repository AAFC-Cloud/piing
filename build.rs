use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=piing.ico");

    if !Path::new("piing.ico").exists() {
        println!(
            "cargo:warning=piing.ico not found; skipping icon embedding. Run make-icon.ps1 to generate it."
        );
        return;
    }

    embed_resource::compile("app.rc", embed_resource::NONE)
        .manifest_required()
        .expect("failed to embed resources");
}
