fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = prost_build::Config::new();

    config.out_dir(std::path::PathBuf::from(
        std::env::var("OUT_DIR").unwrap(),
    ));

    config
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .enum_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");

    config.compile_protos(
        &["proto/mves.proto", "boundary_invariants/contracts.proto"],
        &["."],
    )?;

    println!("cargo:rerun-if-changed=proto/mves.proto");
    println!("cargo:rerun-if-changed=boundary_invariants/contracts.proto");

    Ok(())
}