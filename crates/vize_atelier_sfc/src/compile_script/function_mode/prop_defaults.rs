//! Runtime prop contracts shared by function-mode emission.

use vize_carton::{FxHashMap, String};

use crate::compile_script::props::extract_with_defaults_defaults;
use crate::script::ScriptCompileContext;

pub(super) struct RuntimePropDefaults {
    values: FxHashMap<String, String>,
}

impl RuntimePropDefaults {
    pub(super) fn new(context: &ScriptCompileContext) -> Self {
        Self {
            values: context
                .macros
                .with_defaults
                .as_ref()
                .map(|call| extract_with_defaults_defaults(&call.args))
                .unwrap_or_default(),
        }
    }

    pub(super) fn emit_contract(
        &self,
        output: &mut vize_carton::Vec<u8>,
        name: &str,
        optional: bool,
    ) {
        output.extend_from_slice(if optional {
            b", required: false"
        } else {
            b", required: true"
        });
        if let Some(default) = self.values.get(name) {
            output.extend_from_slice(b", default: ");
            output.extend_from_slice(default.as_bytes());
        }
    }
}
