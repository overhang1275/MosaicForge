use std::{env, fs, path::PathBuf};

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("../../../../contracts/engine/v1");
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    for name in ["generation", "result"] {
        let path = root.join(format!("{name}.schema.json"));
        println!("cargo:rerun-if-changed={}", path.display());
        let schema = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let mut types = typify::TypeSpace::default();
        types.add_root_schema(schema).unwrap();
        let syntax = syn::parse2(types.to_stream()).unwrap();
        fs::write(
            out.join(format!("{name}.rs")),
            prettyplease::unparse(&syntax),
        )
        .unwrap();
    }
    for name in ["job", "result", "page"] {
        let path = root.join(format!("../pdf/v1/{name}.schema.json"));
        println!("cargo:rerun-if-changed={}", path.display());
        let mut value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        let mut settings = typify::TypeSpaceSettings::default();
        for (field, file, title, rust) in [
            (
                "generation",
                "generation",
                "GenerationSpec",
                "crate::contract::GenerationSpec",
            ),
            (
                "variant",
                "result",
                "VariantResult",
                "crate::contract::VariantResult",
            ),
        ] {
            if value["properties"].get(field).is_some() {
                let mut referenced: serde_json::Value = serde_json::from_str(
                    &fs::read_to_string(root.join(format!("{file}.schema.json"))).unwrap(),
                )
                .unwrap();
                let definitions = referenced
                    .as_object_mut()
                    .unwrap()
                    .remove("definitions")
                    .unwrap();
                value["definitions"] = definitions;
                value["definitions"][title] = referenced;
                value["properties"][field] =
                    serde_json::json!({"$ref":format!("#/definitions/{title}")});
                settings.with_replacement(title, rust, std::iter::empty());
            }
        }
        let mut types = typify::TypeSpace::new(&settings);
        types
            .add_root_schema(serde_json::from_value(value).unwrap())
            .unwrap();
        let syntax = syn::parse2(types.to_stream()).unwrap();
        fs::write(
            out.join(format!("pdf_{name}.rs")),
            prettyplease::unparse(&syntax),
        )
        .unwrap();
    }
}
