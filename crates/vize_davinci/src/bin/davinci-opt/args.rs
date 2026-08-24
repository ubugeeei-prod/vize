//! Argument surface of `davinci-opt`: the two modes, the P2-13 pipeline
//! extras, and every usage rejection. Messages here are contract - the CLI
//! tests assert them exactly.

use vize_s0::{String, cstr};

pub enum Mode {
    Roundtrip(std::path::PathBuf),
    Pipeline(String),
}

pub struct Args {
    pub mode: Mode,
    pub stage: String,
    pub folio_dir: Option<std::path::PathBuf>,
    pub folio_after_change: bool,
    pub timing_json: Option<std::path::PathBuf>,
}

pub fn parse_args() -> Result<Args, String> {
    let mut roundtrip = None;
    let mut pipeline = None;
    let mut stage = None;
    let mut folio_dir = None;
    let mut folio_after_change = false;
    let mut timing_json = None;
    let mut argv = std::env::args().skip(1);
    while let Some(arg) = argv.next() {
        match arg.as_str() {
            "--roundtrip" => {
                let value = argv
                    .next()
                    .ok_or_else(|| cstr!("--roundtrip needs a file argument"))?;
                roundtrip = Some(std::path::PathBuf::from(value));
            }
            "--pipeline" => {
                let value = argv
                    .next()
                    .ok_or_else(|| cstr!("--pipeline needs a pipeline string"))?;
                pipeline = Some(String::from(value.as_str()));
            }
            "--stage" => {
                let value = argv
                    .next()
                    .ok_or_else(|| cstr!("--stage needs a stage name"))?;
                stage = Some(String::from(value.as_str()));
            }
            "--folio-dir" => {
                let value = argv
                    .next()
                    .ok_or_else(|| cstr!("--folio-dir needs a directory argument"))?;
                folio_dir = Some(std::path::PathBuf::from(value));
            }
            "--folio-after-change" => folio_after_change = true,
            "--timing-json" => {
                let value = argv
                    .next()
                    .ok_or_else(|| cstr!("--timing-json needs a file argument"))?;
                timing_json = Some(std::path::PathBuf::from(value));
            }
            "--help" | "-h" => return Err(String::default()),
            other => return Err(cstr!("unknown argument: {other}")),
        }
    }
    let mode = match (roundtrip, pipeline) {
        (Some(_), Some(_)) => {
            return Err(cstr!(
                "--roundtrip and --pipeline are alternatives, give exactly one"
            ));
        }
        (Some(path), None) => Mode::Roundtrip(path),
        (None, Some(syntax)) => Mode::Pipeline(syntax),
        (None, None) => {
            return Err(cstr!(
                "--roundtrip <file> or --pipeline \"<syntax>\" is required"
            ));
        }
    };
    // The dump and timing controls are properties of a pipeline run; in
    // roundtrip mode there is no run to dump, so accepting them there would
    // be a silently dead flag.
    if matches!(mode, Mode::Roundtrip(_)) {
        if folio_dir.is_some() {
            return Err(cstr!("--folio-dir requires --pipeline"));
        }
        if timing_json.is_some() {
            return Err(cstr!("--timing-json requires --pipeline"));
        }
    }
    if folio_after_change && folio_dir.is_none() {
        return Err(cstr!("--folio-after-change requires --folio-dir"));
    }
    Ok(Args {
        mode,
        stage: stage.unwrap_or_else(|| String::from("croquis")),
        folio_dir,
        folio_after_change,
        timing_json,
    })
}
