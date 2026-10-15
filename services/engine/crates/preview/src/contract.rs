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
