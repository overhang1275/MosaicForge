mod schema_bundle;

use std::{env, fs, path::PathBuf};

fn main() {
    let root =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../../../../contracts");
    println!("cargo:rerun-if-changed={}", root.display());
    println!("cargo:rerun-if-changed=schema_bundle.rs");
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    for name in [
        "inspect-request",
        "inspect-response",
        "generation-request",
        "generation-response",
        "error",
    ] {
        let schema = schema_bundle::bundle_schema(
            &root,
            &root.join(format!("preview/v1/{name}.schema.json")),
        )
        .expect("local preview schema");
        let mut settings = typify::TypeSpaceSettings::default();
        settings.with_replacement(
            "GenerationSpec",
            "mosaicforge_core::contract::GenerationSpec",
            std::iter::empty(),
        );
        settings.with_replacement(
            "VariantResult",
            "mosaicforge_core::contract::VariantResult",
            std::iter::empty(),
        );
        let mut types = typify::TypeSpace::new(&settings);
        types
            .add_root_schema(serde_json::from_value(schema).unwrap())
            .unwrap();
        let syntax = syn::parse2(types.to_stream()).unwrap();
        fs::write(
            out.join(format!("{}.rs", name.replace('-', "_"))),
            prettyplease::unparse(&syntax),
        )
        .unwrap();
    }
}
