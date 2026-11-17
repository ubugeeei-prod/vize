//! `registerPluginPlan`: the Vite host's one plan registration (#13074).

#![expect(
    clippy::disallowed_types,
    reason = "N-API values cross the boundary as std `String`s"
)]

use napi::{Error, Result, Status};
use napi_derive::napi;
use vize_l0::cstr;

/// Verify one projected `{ digest, plan }` (JSON) and keep it for later
/// compiles. Returns the digest compile options name it by.
#[napi(js_name = "registerPluginPlan")]
pub fn register_plugin_plan(projected: String) -> Result<String> {
    crate::plugin_plans::register(&projected)
        .map(|digest| String::from(digest.as_str()))
        .map_err(refused)
}

/// The directive table one napi compile applies.
pub(crate) fn compile_directives(
    digest: Option<&str>,
    ssr: bool,
    vapor: bool,
) -> Result<vize_l0::config::DirectiveContracts> {
    crate::plugin_plans::compile_directives(digest, ssr, vapor).map_err(refused)
}

fn refused(error: impl core::fmt::Display) -> Error {
    Error::new(Status::InvalidArg, cstr!("{error}").as_str())
}
