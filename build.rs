use std::{env, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=public/style/index.css");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let status = Command::new("bunx")
        .current_dir(manifest_dir)
        .args([
            "lightningcss-cli",
            "-m",
            "public/style/index.css",
            "-o",
            "public/style/index.min.css",
        ])
        .status()
        .expect("failed to execute bunx (is Bun installed?)");

    if !status.success() {
        panic!("bunx lightningcss-cli failed with status: {status}");
    }
}
