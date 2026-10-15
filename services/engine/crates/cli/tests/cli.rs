use mosaicforge_core::template::import_svg;
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
const SVG:&[u8]=br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="80" viewBox="-10 -20 100 80"><g id="layer"><rect x="-10" y="-20" width="100" height="80" fill="#123456"/></g></svg>"##;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    dir: PathBuf,
    path: PathBuf,
}
impl Fixture {
    fn new(data: &[u8]) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "mosaicforge-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
        let path = dir.join("semilla con espacios ñ.svg");
        fs::write(&path, data).unwrap();
        Self { dir, path }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        for entry in fs::read_dir(&self.dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                fs::remove_file(path).unwrap();
            }
        }
        fs::remove_dir(&self.dir).unwrap();
    }
}
fn cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_mosaicforge"))
}
#[test]
fn inspect_reports_stable_normalized_metadata_with_os_paths() {
    let f = Fixture::new(SVG);
    let expected = import_svg(SVG).unwrap();
    let out = cmd()
        .arg("inspect")
        .arg("--template")
        .arg(&f.path)
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    assert!(out.stderr.is_empty());
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(value["templateHash"], expected.hash);
    assert_eq!(value["width"], 100.0);
    assert_eq!(value["height"], 80.0);
    assert_eq!(
        value["viewBox"],
        serde_json::json!([-10.0, -20.0, 100.0, 80.0])
    );
    assert_eq!(value["layerIds"], serde_json::json!(["layer"]));
    assert_eq!(value["segmentCount"], expected.segment_count);
    assert!(value.get("normalizedSvg").is_none());
    assert_eq!(
        out.stdout,
        cmd()
            .arg("inspect")
            .arg("--template")
            .arg(&f.path)
            .output()
            .unwrap()
            .stdout
    );
    assert_eq!(fs::read(&f.path).unwrap(), SVG);
}
#[test]
fn help_version_and_invalid_arguments_have_expected_exit_codes() {
    assert!(cmd().arg("--help").output().unwrap().status.success());
    assert!(cmd().arg("--version").output().unwrap().status.success());
    for args in [
        vec![],
        vec!["inspect"],
        vec!["inspect", "--unknown"],
        vec!["inspect", "--template", "a", "--template", "b"],
        vec!["inspect", "--template"],
    ] {
        let out = cmd().args(args).output().unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        assert!(!out.stderr.is_empty());
    }
}
#[test]
fn unsafe_malformed_and_oversized_inputs_never_emit_success_metadata() {
    for data in [
        b"<svg><script/></svg>".to_vec(),
        b"private-content-not-xml".to_vec(),
        vec![b'x'; 5 * 1024 * 1024 + 1],
    ] {
        let f = Fixture::new(&data);
        let out = cmd()
            .arg("inspect")
            .arg("--template")
            .arg(&f.path)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        let err: Value = serde_json::from_slice(&out.stderr).unwrap();
        assert_eq!(err["error"]["code"], "invalid_svg");
        assert!(!String::from_utf8_lossy(&out.stderr).contains("private-content-not-xml"));
    }
}
#[test]
fn file_errors_have_io_exit_code() {
    let f = Fixture::new(SVG);
    for path in [f.dir.join("missing.svg"), f.dir.clone()] {
        let out = cmd()
            .arg("inspect")
            .arg("--template")
            .arg(path)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(3));
        assert!(out.stdout.is_empty());
        let err: Value = serde_json::from_slice(&out.stderr).unwrap();
        assert_eq!(err["error"]["code"], "io_error");
    }
}

#[test]
fn exact_svg_byte_limit_is_accepted_before_rejecting_the_next_byte() {
    let mut bytes = SVG.to_vec();
    bytes.resize(mosaicforge_core::template::MAX_SVG_BYTES, b' ');
    let f = Fixture::new(&bytes);
    let output = cmd()
        .arg("inspect")
        .arg("--template")
        .arg(&f.path)
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    let parsed: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(parsed["templateHash"], import_svg(SVG).unwrap().hash);
}
