pub mod generation {
    include!(concat!(env!("OUT_DIR"), "/generation.rs"));
}

pub mod result {
    include!(concat!(env!("OUT_DIR"), "/result.rs"));
}

pub use generation::{GenerationSpec, OutputSpec};
pub use result::VariantResult;

pub mod pdf_job {
    include!(concat!(env!("OUT_DIR"), "/pdf_job.rs"));
}
pub mod pdf_result {
    include!(concat!(env!("OUT_DIR"), "/pdf_result.rs"));
}
pub mod pdf_page {
    include!(concat!(env!("OUT_DIR"), "/pdf_page.rs"));
}
pub use pdf_job::PdfJobSpec;
pub use pdf_page::PdfPageRecord;
pub use pdf_result::PdfDocumentResult;
