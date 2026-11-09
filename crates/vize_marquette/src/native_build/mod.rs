//! Typed native build profiles shared by local and hosted executors.

mod model;
mod validate;

#[cfg(test)]
mod tests;

pub use model::{
    NATIVE_BUILD_PROFILE_FORMAT, NATIVE_BUILD_PROFILE_FORMAT_VERSION, NativeBuildConfiguration,
    NativeBuildDestination, NativeBuildPlatform, NativeBuildProfile, NativeBuildProfileDiagnostic,
    NativeBuildProfileDiagnosticCode, NativeBuildSigning, NativeBuildTarget,
    NativeBuildUpdateSelection,
};
pub use validate::validate_native_build_profile;
