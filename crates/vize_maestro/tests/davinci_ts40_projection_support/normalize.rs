use sha2::{Digest, Sha256};
use std::fmt::Write;
use vize_carton::{SmallVec, String, ToCompactString};

use super::matrix::Fixture;

pub(super) type Lines = SmallVec<[String; 8]>;

pub(super) fn sha256(text: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(text.as_bytes());
    let mut output = String::with_capacity(64);
    for byte in hash.finalize() {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

pub(super) fn stable_lines(mut lines: Lines) -> String {
    lines.sort_unstable();
    let mut output = String::default();
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            output.push('\n');
        }
        output.push_str(line);
    }
    output
}

pub(super) fn fixture_path(fixture: &Fixture) -> &std::path::Path {
    std::path::Path::new(fixture.file.as_str())
}

pub(super) fn normalized_error(error: impl core::fmt::Display) -> String {
    error.to_compact_string().replace('\\', "/").into()
}
