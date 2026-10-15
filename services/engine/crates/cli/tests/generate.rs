use mosaicforge_core::{contract::GenerationSpec, template::import_svg};
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><g><rect width="100" height="100" fill="#123456"/><rect x="25" y="25" width="20" height="30" fill="#fedcba"/></g></svg>"##;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Job {
    dir: PathBuf,
    spec: Value,
}
impl Job {
    fn new(format: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "mosaicforge-generate-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("seed ñ.svg"), SVG).unwrap();
        let mut spec: Value =
            serde_json::from_str(include_str!("../../../tests/fixtures/identity.json")).unwrap();
        spec["templateHash"] = json!(import_svg(SVG).unwrap().hash);
        spec["count"] = json!(3);
        spec["startIndex"] = json!("9");
        spec["output"] = json!({"format":format,"width":24,"height":24,"background":"transparent"});
        Self { dir, spec }
    }
    fn command(&self, name: &str) -> Command {
        fs::write(
            self.dir.join("spec.json"),
            serde_json::to_vec(&self.spec).unwrap(),
        )
        .unwrap();
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_mosaicforge"));
        cmd.arg("generate")
            .arg("--template")
            .arg(self.dir.join("seed ñ.svg"))
            .arg("--spec")
            .arg(self.dir.join("spec.json"))
            .arg("--out")
            .arg(self.dir.join(name));
        cmd
    }
    fn run(&self, name: &str) -> Output {
        self.command(name).output().unwrap()
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        // Only remove this fixture's own files, one directory level at a time.
        for entry in fs::read_dir(&self.dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                for child in fs::read_dir(&path).unwrap() {
                    fs::remove_file(child.unwrap().path()).unwrap();
                }
                fs::remove_dir(path).unwrap();
            } else {
                fs::remove_file(path).unwrap();
            }
        }
        fs::remove_dir(&self.dir).unwrap();
    }
}
fn records(output: &Output) -> Vec<Value> {
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty());
    String::from_utf8(output.stdout.clone())
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
}
#[test]
fn svg_and_png_match_cpu_bytes_and_results_at_global_indices() {
    // Bypassing the CPU pipeline or resetting the index breaks this comparison.
    for format in ["svg", "png"] {
        let job = Job::new(format);
        let got = records(&job.run("out"));
        let template = import_svg(SVG).unwrap();
        let spec: GenerationSpec = serde_json::from_value(job.spec.clone()).unwrap();
        for (offset, index) in (9..12).enumerate() {
            let expected = mosaicforge_cpu::export::generate_and_export(
                &template,
                &spec,
                index,
                mosaicforge_cpu::render::MAX_RENDER_BUFFER_BYTES,
            )
            .unwrap();
            assert_eq!(
                got[offset],
                json!({"kind":"variant","result":expected.variant.result})
            );
            assert_eq!(
                fs::read(
                    job.dir
                        .join("out")
                        .join(format!("variant-{index}.{format}"))
                )
                .unwrap(),
                expected.artifact.unwrap()
            );
        }
        assert_eq!(got[3], json!({"kind":"summary","accepted":3,"rejected":0}));
        assert_eq!(fs::read_dir(job.dir.join("out")).unwrap().count(), 3);
        assert_eq!(fs::read(job.dir.join("seed ñ.svg")).unwrap(), SVG);
    }
}
#[test]
fn existing_output_is_never_replaced() {
    let job = Job::new("svg");
    records(&job.run("out"));
    let target = job.dir.join("out/variant-9.svg");
    fs::write(&target, b"keep existing bytes").unwrap();
    let output = job.run("out");
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(fs::read(target).unwrap(), b"keep existing bytes");
    assert_eq!(fs::read_dir(job.dir.join("out")).unwrap().count(), 3);
}
#[test]
fn invalid_spec_and_wrong_template_hash_create_no_output_directory() {
    for bad in [
        json!("bad hash"),
        json!("0000000000000000000000000000000000000000000000000000000000000000"),
    ] {
        let mut job = Job::new("svg");
        job.spec["templateHash"] = bad;
        let output = job.run("out");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let error: Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(error["error"]["code"], "invalid_spec");
        assert!(!job.dir.join("out").exists());
    }
}
#[test]
fn oversized_malformed_and_missing_spec_are_rejected_before_publication() {
    let job = Job::new("svg");
    for bytes in [vec![b'x'; 1024 * 1024 + 1], b"private-json-source".to_vec()] {
        let mut command = job.command("out");
        fs::write(job.dir.join("spec.json"), bytes).unwrap();
        let output = command.output().unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("private-json-source"));
        assert!(!job.dir.join("out").exists());
    }
    let mut command = job.command("out");
    fs::remove_file(job.dir.join("spec.json")).unwrap();
    assert_eq!(command.output().unwrap().status.code(), Some(3));
    assert!(!job.dir.join("out").exists());
}
#[test]
fn valid_rejections_have_success_status_and_no_artifacts() {
    let mut job = Job::new("png");
    job.spec["rules"]["opacity"] = json!({"min":0,"max":0});
    let got = records(&job.run("out"));
    for value in &got[..3] {
        assert_eq!(value["result"]["status"], "rejected");
        assert!(value["result"]["artifactHash"].is_null());
    }
    assert_eq!(got[3], json!({"kind":"summary","accepted":0,"rejected":3}));
    assert_eq!(fs::read_dir(job.dir.join("out")).unwrap().count(), 0);
}

#[test]
fn groups_threads_and_subranges_preserve_bytes_and_order_for_201_variants() {
    // Resetting RNG/index at a boundary or using scheduling order changes this output.
    for format in ["svg", "png"] {
        let mut job = Job::new(format);
        job.spec["count"] = json!(201);
        job.spec["rules"]["scaleX"] = json!({"min":1.0,"max":1.3});
        job.spec["rules"]["scaleY"] = json!({"min":1.0,"max":1.3});
        job.spec["rules"]["rotationDegrees"] = json!({"min":0.0,"max":90.0});
        let baseline = job.run("reference");
        let values = records(&baseline);
        assert_eq!(values.len(), 202);
        assert!(values[201]["accepted"].as_u64().unwrap() > 0);
        let hashes: std::collections::BTreeSet<_> = values[..201]
            .iter()
            .filter_map(|v| v["result"]["artifactHash"].as_str())
            .collect();
        assert!(
            hashes.len() > 1,
            "fixture must generate distinct accepted artifacts"
        );
        for (n, threads) in [(1, 1), (7, 1), (100, 1), (1, 2), (7, 2), (100, 2)] {
            let name = format!("n{n}-t{threads}");
            let output = job
                .command(&name)
                .args([
                    "--batch-size",
                    &n.to_string(),
                    "--threads",
                    &threads.to_string(),
                ])
                .output()
                .unwrap();
            records(&output);
            assert_eq!(
                fs::read_dir(job.dir.join(&name)).unwrap().count() as u64,
                values[201]["accepted"].as_u64().unwrap()
            );
            assert_eq!(
                output.stdout, baseline.stdout,
                "{format}, N={n}, threads={threads}"
            );
            for index in 9..210 {
                let filename = format!("variant-{index}.{format}");
                let expected = fs::read(job.dir.join("reference").join(&filename));
                let actual = fs::read(job.dir.join(&name).join(&filename));
                match (expected, actual) {
                    (Ok(a), Ok(b)) => assert_eq!(a, b, "{filename}"),
                    (Err(a), Err(b)) => {
                        assert_eq!(a.kind(), std::io::ErrorKind::NotFound);
                        assert_eq!(b.kind(), std::io::ErrorKind::NotFound);
                    }
                    _ => panic!("artifact presence differs for {filename}"),
                }
            }
        }
        job.spec["startIndex"] = json!("108");
        job.spec["count"] = json!(7);
        let subrange = records(
            &job.command("range")
                .args(["--batch-size", "7", "--threads", "2"])
                .output()
                .unwrap(),
        );
        assert_eq!(subrange[..7], values[99..106]);
        for index in 108..115 {
            let filename = format!("variant-{index}.{format}");
            let expected = fs::read(job.dir.join("reference").join(&filename));
            let actual = fs::read(job.dir.join("range").join(filename));
            assert_eq!(actual.ok(), expected.ok());
        }
    }
}

#[test]
fn highest_u64_index_is_supported_without_exclusive_end_overflow() {
    let mut job = Job::new("svg");
    job.spec["startIndex"] = json!(u64::MAX.to_string());
    job.spec["count"] = json!(1);
    let output = records(
        &job.command("last")
            .args(["--batch-size", "100", "--threads", "2"])
            .output()
            .unwrap(),
    );
    assert_eq!(output[0]["result"]["index"], u64::MAX.to_string());
    assert!(
        job.dir
            .join(format!("last/variant-{}.svg", u64::MAX))
            .is_file()
    );
}
#[test]
fn invalid_group_and_thread_limits_fail_before_writing() {
    let job = Job::new("png");
    for args in [
        ["--batch-size", "0"],
        ["--batch-size", "10001"],
        ["--threads", "0"],
        ["--threads", "3"],
    ] {
        let output = job.command("out").args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!job.dir.join("out").exists());
    }
}
