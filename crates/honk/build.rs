use std::error::Error;
use std::path::PathBuf;
use std::{env, fs, io};

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::other("CARGO_MANIFEST_DIR is not set"))?;
    let hoon_source = manifest_dir.join("../../hoon/common/hoon.hoon");
    let honc_type_asset = manifest_dir.join("assets/honc-type-138.jam");
    let honc_formula_asset = manifest_dir.join("assets/honc-formula-138.jam");
    let hoonc_octs_type_asset = manifest_dir.join("assets/hoonc-octs-type-138.jam");

    let hoonc_octs_type_asset_path = if hoonc_octs_type_asset.exists() {
        hoonc_octs_type_asset.clone()
    } else {
        let out_dir = env::var_os("OUT_DIR")
            .map(PathBuf::from)
            .ok_or_else(|| io::Error::other("OUT_DIR is not set"))?;
        let placeholder = out_dir.join("hoonc-octs-type-138.missing.jam");
        fs::write(&placeholder, [])?;
        println!(
            "cargo:warning=assets/hoonc-octs-type-138.jam is missing; using an empty compile-time placeholder. Data-import compilation will require `bazel build //crates/honk:hoonc_octs_type_138` or `make build-honk-assets`."
        );
        placeholder
    };

    for path in [&hoon_source, &honc_type_asset, &honc_formula_asset, &hoonc_octs_type_asset] {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!(
        "cargo:rustc-env=HONK_HOON_138_SOURCE={}",
        hoon_source.display()
    );
    println!(
        "cargo:rustc-env=HONK_HONC_TYPE_138_JAM={}",
        honc_type_asset.display()
    );
    println!(
        "cargo:rustc-env=HONK_HONC_FORMULA_138_JAM={}",
        honc_formula_asset.display()
    );
    println!(
        "cargo:rustc-env=HONK_HOONC_OCTS_TYPE_138_JAM={}",
        hoonc_octs_type_asset_path.display()
    );
    Ok(())
}
