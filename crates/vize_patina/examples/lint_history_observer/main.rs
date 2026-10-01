//! Complete public Patina observations for the shared fix-history corpus.
//! The legacy observer provides no native implementation or product route.

#![expect(
    clippy::disallowed_macros,
    clippy::panic,
    reason = "fixture observers preserve complete actual Debug output and fail on invalid inputs"
)]

mod current_api;
mod report;
mod static_class;

use std::io::{self, Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [api] = args.as_slice() else {
        return Err(
            io::Error::new(io::ErrorKind::InvalidInput, "expected one observer API").into(),
        );
    };
    if api == "--contract" {
        writeln!(
            io::stdout().lock(),
            "{{\"schema\":\"vize.linter-history-observer\",\"version\":1,\"apis\":[\"--current-api\",\"--report\",\"--static-class\"],\"preset\":\"Incremental\",\"locale\":\"En\",\"help\":\"Full\",\"native\":\"unsupported\"}}"
        )?;
        return Ok(());
    }
    let mut bytes = Vec::new();
    io::stdin().read_to_end(&mut bytes)?;
    let input = std::str::from_utf8(&bytes)?;
    let observe = match api.as_str() {
        "--current-api" => current_api::capture,
        "--report" => report::capture,
        "--static-class" => static_class::capture,
        _ => {
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput, "unsupported observer API").into(),
            );
        }
    };
    let first = observe(input)?;
    let repeat = observe(input)?;
    assert_eq!(
        first, repeat,
        "complete observation changed within the same process"
    );
    io::stdout().lock().write_all(first.as_bytes())?;
    Ok(())
}
