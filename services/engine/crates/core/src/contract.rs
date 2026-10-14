pub mod generation {
    include!(concat!(env!("OUT_DIR"), "/generation.rs"));
}

pub mod result {
    include!(concat!(env!("OUT_DIR"), "/result.rs"));
}

pub use generation::{GenerationSpec, OutputSpec};
pub use result::VariantResult;
