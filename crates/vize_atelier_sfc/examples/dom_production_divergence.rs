//! Reducer predicate: exits 0 when a production DOM compile shape emits a
//! module that differs from the pinned legacy lane (the selected lane, or the
//! Croquis projection the production-reach gate also holds to parity), so
//! `vize reduce --script` keeps exactly what that gate counts as a
//! divergence.
//!
//! Usage: `dom_production_divergence --shape dom_inline|dom_module [--name NAME] FILE`.
//! `NAME` is the corpus-relative filename the gate compiles under (default:
//! `FILE`). Exit 1 means no difference or no template; exit 2 is a usage or
//! read error.

use std::process::ExitCode;

use vize_atelier_sfc::dom_differential::diff::production_divergence;
use vize_atelier_sfc::dom_differential::shapes::Shape;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};

fn usage() -> ExitCode {
    eprintln!("usage: dom_production_divergence --shape dom_inline|dom_module [--name NAME] FILE");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (shape, name, file) = match args.as_slice() {
        [flag, shape, file] if flag == "--shape" => (shape, file, file),
        [flag, shape, name_flag, name, file] if flag == "--shape" && name_flag == "--name" => {
            (shape, name, file)
        }
        _ => return usage(),
    };
    let shape = match shape.as_str() {
        "dom_inline" => Shape::DomInline,
        "dom_module" => Shape::DomModule,
        _ => return usage(),
    };
    let Ok(source) = std::fs::read_to_string(file) else {
        eprintln!("dom_production_divergence: cannot read {file}");
        return ExitCode::from(2);
    };
    let options = SfcParseOptions {
        filename: name.as_str().into(),
        ..Default::default()
    };
    let Ok(descriptor) = parse_sfc(&source, options) else {
        return ExitCode::FAILURE;
    };
    if descriptor.template.is_none() {
        return ExitCode::FAILURE;
    }
    if production_divergence(&descriptor, name, shape).is_some() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
