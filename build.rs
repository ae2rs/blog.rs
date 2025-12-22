use std::env;
use std::path::PathBuf;
use std::process::Command;

fn tailwind_binary_name(os: &str, arch: &str) -> Option<&'static str> {
    match (os, arch) {
        ("macos", "aarch64") => Some("tailwindcss-macos-arm64"),
        ("macos", "x86_64") => Some("tailwindcss-macos-x64"),
        ("linux", "aarch64") => Some("tailwindcss-linux-arm64"),
        ("linux", "x86_64") => Some("tailwindcss-linux-x64"),
        _ => None,
    }
}

fn main() {
    println!("cargo:rerun-if-changed=styles/index.css");
    println!("cargo:rerun-if-changed=tailwind.config.js");
    println!("cargo:rerun-if-changed=src");

    let os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "unknown".to_string());
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "unknown".to_string());
    let bin_name = tailwind_binary_name(&os, &arch).unwrap_or_else(|| {
        panic!(
            "unsupported target for tailwindcss binary: os={}, arch={}",
            os, arch
        )
    });

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let bin_path = manifest_dir.join("vendor").join("tailwind").join(bin_name);
    let input = manifest_dir.join("styles").join("index.css");
    let output = manifest_dir.join("public").join("style").join("index.css");

    let status = Command::new(&bin_path)
        .current_dir(&manifest_dir)
        .arg("-i")
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("--minify")
        .status()
        .unwrap_or_else(|err| {
            panic!(
                "failed to execute tailwindcss binary at {}: {}",
                bin_path.display(),
                err
            )
        });

    if !status.success() {
        panic!("tailwindcss build failed with status: {}", status);
    }
}
