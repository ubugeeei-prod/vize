//! NAPI binding for the Rust CLI entrypoint.
// `#[napi]` generates the argv conversion next to this function, so the
// expectation covers the module rather than the item.
#![expect(
    clippy::disallowed_types,
    reason = "N-API passes argv as std `String`s"
)]

use napi::Result;
use napi_derive::napi;

#[napi(js_name = "runCli")]
pub fn run_cli(args: Vec<String>) -> Result<()> {
    vize::cli::run_from_args(args);
    Ok(())
}
