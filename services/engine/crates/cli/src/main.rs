use clap::{Arg, Command, value_parser};
use mosaicforge_core::template::{MAX_SVG_BYTES, Template, import_svg};
use serde_json::json;
use std::{
    ffi::OsString,
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};
mod generation;
mod publish;

struct Failure {
    exit: u8,
    code: &'static str,
    message: String,
}
fn io_failure(message: &'static str) -> Failure {
    Failure {
        exit: 3,
        code: "io_error",
        message: message.into(),
    }
}
fn invalid_svg(message: impl Into<String>) -> Failure {
    Failure {
        exit: 2,
        code: "invalid_svg",
        message: message.into(),
    }
}

fn main() -> ExitCode {
    let code = run(
        std::env::args_os(),
        &mut io::stdout().lock(),
        &mut io::stderr().lock(),
    );
    ExitCode::from(code)
}
fn run(args: impl IntoIterator<Item = OsString>, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let command = Command::new("mosaicforge")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Deterministic CPU vector generation engine")
        .subcommand_required(true)
        .subcommand(
            Command::new("inspect")
                .about("Inspect a supported SVG without changing it")
                .arg(
                    Arg::new("template")
                        .long("template")
                        .value_name("FILE")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                ),
        )
        .subcommand(
            Command::new("generate")
                .about("Generate SVG/PNG variants without replacing existing files")
                .args(["template", "spec", "out"].map(|name| {
                    Arg::new(name)
                        .long(name)
                        .value_name("PATH")
                        .required(true)
                        .value_parser(value_parser!(PathBuf))
                })),
        );
    let matches = match command.try_get_matches_from(args) {
        Ok(m) => m,
        Err(e) => {
            if !e.use_stderr() {
                return if out
                    .write_all(e.to_string().as_bytes())
                    .and_then(|_| out.flush())
                    .is_ok()
                {
                    0
                } else {
                    report(io_failure("cannot write command output"), err)
                };
            }
            return report(
                Failure {
                    exit: 2,
                    code: "invalid_arguments",
                    message: e.to_string(),
                },
                err,
            );
        }
    };
    if let Some(("generate", arguments)) = matches.subcommand() {
        return match generation::run(arguments, out) {
            Ok(()) => 0,
            Err(e) => report(e, err),
        };
    }
    let Some(("inspect", arguments)) = matches.subcommand() else {
        return report(
            Failure {
                exit: 2,
                code: "invalid_arguments",
                message: "expected inspect command".into(),
            },
            err,
        );
    };
    let Some(path) = arguments.get_one::<PathBuf>("template") else {
        return report(
            Failure {
                exit: 2,
                code: "invalid_arguments",
                message: "template path is required".into(),
            },
            err,
        );
    };
    let template = match read_template(path) {
        Ok(t) => t,
        Err(e) => return report(e, err),
    };
    let value = json!({"templateHash":template.hash,"width":template.width,"height":template.height,"viewBox":template.view_box,"layerIds":template.layer_ids,"segmentCount":template.segment_count});
    if serde_json::to_writer(&mut *out, &value).is_err()
        || out.write_all(b"\n").and_then(|_| out.flush()).is_err()
    {
        return report(io_failure("cannot write command output"), err);
    }
    0
}
fn report(failure: Failure, err: &mut impl Write) -> u8 {
    let value = json!({"error":{"code":failure.code,"message":failure.message}});
    let _ = serde_json::to_writer(&mut *err, &value);
    let _ = err.write_all(b"\n");
    let _ = err.flush();
    failure.exit
}
fn read_template(path: &Path) -> Result<Template, Failure> {
    let metadata =
        std::fs::metadata(path).map_err(|_| io_failure("cannot inspect template file"))?;
    if !metadata.is_file() {
        return Err(io_failure("template must be a regular file"));
    }
    let file = File::open(path).map_err(|_| io_failure("cannot open template file"))?;
    if !file
        .metadata()
        .map_err(|_| io_failure("cannot inspect open template file"))?
        .is_file()
    {
        return Err(io_failure("template must be a regular file"));
    }
    let bytes = read_bounded(file)?;
    import_svg(&bytes).map_err(|error| invalid_svg(error.to_string()))
}
fn read_bounded(reader: impl Read) -> Result<Vec<u8>, Failure> {
    let mut bytes = Vec::new();
    reader
        .take(MAX_SVG_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| io_failure("cannot read template file"))?;
    if bytes.len() > MAX_SVG_BYTES {
        return Err(invalid_svg("SVG byte limit exceeded"));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_reader_stops_after_limit_plus_one_byte() {
        struct Endless {
            count: usize,
        }
        impl Read for &mut Endless {
            fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
                bytes.fill(b'x');
                self.count += bytes.len();
                Ok(bytes.len())
            }
        }
        let mut reader = Endless { count: 0 };
        assert!(read_bounded(&mut reader).is_err());
        assert_eq!(reader.count, MAX_SVG_BYTES + 1);
    }
    #[test]
    fn output_errors_return_io_status_instead_of_panicking() {
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("injected"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut errors = Vec::new();
        assert_eq!(
            run(
                ["mosaicforge", "--help"].map(OsString::from),
                &mut Broken,
                &mut errors
            ),
            3
        );
        let wire: serde_json::Value = serde_json::from_slice(&errors).unwrap();
        assert_eq!(wire["error"]["code"], "io_error");
    }
}
