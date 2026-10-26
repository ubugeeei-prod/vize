//! Immutable identity carried by the actually linked native image.
#![expect(
    clippy::disallowed_types,
    reason = "N-API returns an owned JavaScript string"
)]

use napi_derive::napi;

/// An opaque identity minted once by the native build invocation.
/// Direct builds without the instrumented producer return no identity.
#[napi(js_name = "getNativeBuildToken")]
pub fn get_native_build_token() -> Option<String> {
    // Release custody also verifies the token's contiguous bytes in cross-built
    // images. Without the barrier, x86 LLVM can synthesize the String with
    // separate immediate constants and remove the original contiguous literal.
    std::hint::black_box(option_env!("VIZE_NATIVE_BUILD_TOKEN")).map(String::from)
}
