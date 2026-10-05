//! Complete public Patina observations for the shared fix-history corpus.
//! Native observations use the opt-in configured original template/SFC routes.

#![expect(
    clippy::disallowed_macros,
    clippy::panic,
    reason = "fixture observers preserve complete actual Debug output and fail on invalid inputs"
)]

mod current_api;
mod native;
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
            "{{\"schema\":\"vize.linter-history-observer\",\"version\":3,\"apis\":[\"--current-api\",\"--report\",\"--static-class\"],\"preset\":\"Incremental\",\"locale\":\"En\",\"help\":\"Full\",\"native\":\"configured-template-and-sfc\",\"nativeApis\":[\"--native-current-api\",\"--native-report\",\"--native-static-class\"]}}"
        )?;
        return Ok(());
    }
    if api == "--native-contract" {
        writeln!(
            io::stdout().lock(),
            "{{\"schema\":\"vize.linter-native-observer\",\"version\":2,\"apis\":[\"--native-current-api\",\"--native-report\",\"--native-static-class\"],\"owners\":{{\"template\":\"NativeLintComponent\",\"sfc\":\"NativeSfcLintOwner\"}},\"entries\":[\"template\",\"sfc\"],\"wholeOutput\":\"Case+Observation\",\"fallback\":false}}"
        )?;
        return Ok(());
    }
    let mut bytes = Vec::new();
    io::stdin().read_to_end(&mut bytes)?;
    let input = std::str::from_utf8(&bytes)?;
    if matches!(
        api.as_str(),
        "--native-current-api" | "--native-report" | "--native-static-class"
    ) {
        let first = native::capture(api, input)?;
        let repeat = native::capture(api, input)?;
        assert_eq!(
            first, repeat,
            "complete native outcome changed within the same process"
        );
        io::stdout().lock().write_all(first.as_bytes())?;
        return Ok(());
    }
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
