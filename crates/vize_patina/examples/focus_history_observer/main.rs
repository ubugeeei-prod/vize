//! Source-qualified first observations; no complete oracle is accepted here.

#![expect(
    clippy::disallowed_macros,
    clippy::panic,
    reason = "capture fixtures fail on invalid inputs and retain complete actual Debug bytes"
)]

mod case;
mod observe;

use std::io::{self, Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [api] = args.as_slice() else {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "expected one capture API").into());
    };
    if api == "--contract" {
        writeln!(
            io::stdout().lock(),
            "{{\"schema\":\"vize.focus-history-observer\",\"version\":1,\"apis\":[\"--legacy\",\"--native\"],\"entry\":\"lint_template\",\"filename\":\"test.vue\",\"registry\":\"one-concrete-original-rule\",\"options\":\"original-unspecified\",\"output\":\"Case+RuleIdentity+Observation\",\"acceptance\":\"capture-only\",\"fallback\":false}}"
        )?;
        return Ok(());
    }
    if !matches!(api.as_str(), "--legacy" | "--native") {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "unsupported capture API").into());
    }
    let mut bytes = Vec::new();
    io::stdin().read_to_end(&mut bytes)?;
    let input = std::str::from_utf8(&bytes)?;
    let first = observe::capture(api, input)?;
    let repeat = observe::capture(api, input)?;
    assert_eq!(
        first, repeat,
        "whole capture changed within the same process"
    );
    io::stdout().lock().write_all(first.as_bytes())?;
    Ok(())
}
