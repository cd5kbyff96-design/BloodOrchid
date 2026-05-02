use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let kernel_file = workspace_root.join("kernel/pde_ref/kernel.cc");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));

    println!("cargo:rerun-if-changed={}", kernel_file.display());

    let compiler = env::var("CXX").unwrap_or_else(|_| "clang++".to_string());
    let kernel_obj = out_dir.join("kernel.o");
    let archive = out_dir.join("libmves_kernel.a");

    run(Command::new(&compiler)
        .arg("-std=c++17")
        .arg("-O0")
        .arg("-c")
        .arg(&kernel_file)
        .arg("-o")
        .arg(&kernel_obj));

    run(Command::new("ar")
        .arg("crus")
        .arg(&archive)
        .arg(&kernel_obj));

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=mves_kernel");
    println!("cargo:rustc-link-lib=dylib=c++");
}

fn run(command: &mut Command) {
    let status = command.status().expect("failed to start command");
    assert!(status.success(), "command failed with status {}", status);
}