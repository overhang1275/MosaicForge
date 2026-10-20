pub mod inspect_request {
    include!(concat!(env!("OUT_DIR"), "/inspect_request.rs"));
}
pub mod inspect_response {
    include!(concat!(env!("OUT_DIR"), "/inspect_response.rs"));
}
pub mod generation_request {
    include!(concat!(env!("OUT_DIR"), "/generation_request.rs"));
}
pub mod generation_response {
    include!(concat!(env!("OUT_DIR"), "/generation_response.rs"));
}
pub mod preview_error {
    include!(concat!(env!("OUT_DIR"), "/error.rs"));
}
pub use generation_request::GenerationRequest;
pub use generation_response::GenerationResponse;
pub use inspect_request::InspectRequest;
pub use inspect_response::InspectResponse;
pub use preview_error::PreviewError;

pub mod pdf_export_request {
    include!(concat!(env!("OUT_DIR"), "/pdf_export_request.rs"));
}
pub mod pdf_export_response {
    include!(concat!(env!("OUT_DIR"), "/pdf_export_response.rs"));
}
pub use pdf_export_request::PdfExportRequest;
pub use pdf_export_response::PdfExportResponse;
pub mod selected_pdf_export_request {
    include!(concat!(env!("OUT_DIR"), "/selected_pdf_export_request.rs"));
}
pub mod selected_pdf_export_response {
    include!(concat!(env!("OUT_DIR"), "/selected_pdf_export_response.rs"));
}
pub use selected_pdf_export_request::SelectedPdfExportRequest;
pub use selected_pdf_export_response::SelectedPdfExportResponse;
pub mod pdf_download_response {
    include!(concat!(env!("OUT_DIR"), "/pdf_download_response.rs"));
}
pub use pdf_download_response::PdfDownloadResponse;
pub mod block_request {
    include!(concat!(env!("OUT_DIR"), "/block_request.rs"));
}
pub mod block_response {
    include!(concat!(env!("OUT_DIR"), "/block_response.rs"));
}
pub use block_request::BlockRequest;
pub use block_response::BlockResponse;
pub mod finalize_request {
    include!(concat!(env!("OUT_DIR"), "/finalize_request.rs"));
}
pub mod finalize_response {
    include!(concat!(env!("OUT_DIR"), "/finalize_response.rs"));
}
pub mod cancel_request {
    include!(concat!(env!("OUT_DIR"), "/cancel_request.rs"));
}
pub mod cancel_response {
    include!(concat!(env!("OUT_DIR"), "/cancel_response.rs"));
}
pub mod manifest_record {
    include!(concat!(env!("OUT_DIR"), "/manifest_record.rs"));
}
