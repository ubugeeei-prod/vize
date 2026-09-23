//! The committed output-world probe.

use std::path::PathBuf;

use vize_extension_host::contract::Page;
use vize_extension_host::output::EmitRequest;
use vize_s0::String;

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/output")
}

fn read(name: &str) -> String {
    let path = dir().join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    String::from_utf8(bytes).expect("output fixtures are UTF-8")
}

fn page(name: &str) -> Page {
    Page {
        schema_version: 1,
        text: read(name),
    }
}

pub fn request() -> EmitRequest {
    EmitRequest {
        s2: page("probe.s2.folio"),
        s3: page("probe.s3.folio"),
    }
}

pub fn document() -> String {
    read("probe.emit.folio")
}
