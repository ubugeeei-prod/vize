//! Owned typed-router authority admitted once for an opened mapper project.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use super::super::tsconfig_gen::compiler_inputs::CompilerInputs;
use super::super::typed_router::{Config, provider};
use crate::batch::error::CorsaResult;

/// Project-owned router capability. Transforming a file never reads its provider.
/// The legacy root-only transform API deliberately retains file-only behavior.
#[derive(Clone, Debug, Default)]
pub struct ContentMapperProjectContext {
    root: Option<PathBuf>,
    route_props: bool,
}

impl ContentMapperProjectContext {
    pub(in crate::batch::virtual_project) fn config(&self) -> Option<Config<'_>> {
        self.root.as_deref().map(|root| Config {
            root,
            route_props: self.route_props,
        })
    }

    #[expect(
        clippy::disallowed_types,
        reason = "host compiler options use serde_json keys"
    )]
    pub(super) fn admit(
        root: Option<PathBuf>,
        config_file_name: &Path,
        compiler: Option<&Map<std::string::String, Value>>,
        watched: &mut Vec<vize_l0::String>,
    ) -> CorsaResult<(Self, Option<[u8; 32]>)> {
        let mut context = Self {
            root,
            route_props: false,
        };
        let Some(compiler) = compiler.filter(|_| context.root.is_some()) else {
            return Ok((context, None));
        };
        // Older callers supply only a root. Preserve their file-only authority
        // without inventing the host's module-resolution defaults.
        if !provider::supports_resolution_context(compiler) {
            return Ok((context, None));
        }
        let inputs = CompilerInputs::default();
        let (admitted, identity) = provider::capture_route_props(
            config_file_name.parent().unwrap_or(Path::new(".")),
            compiler,
            &inputs,
        )?;
        inputs.validate()?;
        watched.extend(
            inputs
                .paths()
                .into_iter()
                .map(|path| path.to_string_lossy().into()),
        );
        context.route_props = admitted;
        Ok((context, Some(identity)))
    }
}
