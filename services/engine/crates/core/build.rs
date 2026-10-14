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
}
