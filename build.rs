use std::env;
use std::fs;
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

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) {
    if !src.exists() {
        return;
    }

    fs::create_dir_all(dst)
        .unwrap_or_else(|err| panic!("failed to create directory {}: {}", dst.display(), err));

    let entries = fs::read_dir(src)
        .unwrap_or_else(|err| panic!("failed to read directory {}: {}", src.display(), err));

    for entry in entries {
        let entry = entry.unwrap_or_else(|err| {
            panic!(
                "failed to read directory entry in {}: {}",
                src.display(),
                err
            )
        });
        let path = entry.path();
        let target = dst.join(entry.file_name());
        let metadata = entry.metadata().unwrap_or_else(|err| {
            panic!("failed to read metadata for {}: {}", path.display(), err)
        });

        if metadata.is_dir() {
            copy_dir_recursive(&path, &target);
        } else if metadata.is_file() {
            fs::copy(&path, &target).unwrap_or_else(|err| {
                panic!(
                    "failed to copy file from {} to {}: {}",
                    path.display(),
                    target.display(),
                    err
                )
            });
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=styles/index.css");
    println!("cargo:rerun-if-changed=tailwind.config.js");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=assets");

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
    let output = manifest_dir.join("build").join("style").join("index.css");
    let output_parent = output.parent().unwrap();
    fs::create_dir_all(output_parent).unwrap_or_else(|err| {
        panic!(
            "failed to create output directory {}: {}",
            output_parent.display(),
            err
        )
    });

    let status = Command::new(&bin_path)
        .current_dir(&manifest_dir)
        .arg("build")
        .arg("--config")
        .arg(manifest_dir.join("tailwind.config.js"))
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

    let assets_dir = manifest_dir.join("assets");
    let build_assets_dir = manifest_dir.join("build");
    copy_dir_recursive(&assets_dir, &build_assets_dir);
}
