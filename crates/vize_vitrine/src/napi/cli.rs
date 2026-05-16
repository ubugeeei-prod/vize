//! NAPI binding for the Rust CLI entrypoint.

use napi::Result;
use napi_derive::napi;

#[napi(js_name = "runCli")]
#[allow(clippy::disallowed_types)]
pub fn run_cli(args: Vec<String>) -> Result<()> {
    vize::cli::run_from_args(args);
    Ok(())
}
