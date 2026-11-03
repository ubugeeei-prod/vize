use vize_l0::{Span, String, cstr};
use vize_l2::file::FileArtifact;
use vize_l3::jsx::{JsxDecisionKind, NativeJsxAnalysis};

use super::{JsxEmitError, JsxEmitErrorKind};
use crate::runtime::{Helper, Runtime, vocabulary};

pub(super) struct SelectedHelper {
    pub id: Helper,
    pub export: &'static str,
    pub module: &'static str,
    pub alias: String,
}

pub(super) struct SelectedHelpers {
    pub node: SelectedHelper,
    pub text: Option<SelectedHelper>,
}

pub(super) fn selected(analysis: &NativeJsxAnalysis<'_>) -> Result<SelectedHelpers, JsxEmitError> {
    let file = analysis.owner().file();
    let node = one(file, "createVNode")?;
    let text = analysis
        .decisions()
        .any(|decision| {
            matches!(decision.kind(), JsxDecisionKind::Text(value) if super::whitespace::has_text(value))
        })
        .then(|| one(file, "createTextVNode"))
        .transpose()?;
    Ok(SelectedHelpers { node, text })
}

fn one(file: &FileArtifact<'_>, name: &str) -> Result<SelectedHelper, JsxEmitError> {
    let runtime = vocabulary(Runtime::VueDom);
    let failure = || JsxEmitError {
        kind: JsxEmitErrorKind::RuntimeHelper,
        span: Span::new(0, 0),
    };
    let id = runtime.helper(name).ok_or_else(failure)?;
    let (module, export) = runtime.export(id).ok_or_else(failure)?;
    for index in 0..=u32::MAX {
        let alias = cstr!("_vize_{name}{index}");
        // Every lexical scope matters: a function parameter can shadow imports.
        // References also prevent capturing an unrelated unresolved global read.
        if file.bindings().any(|binding| {
            binding
                .declaration()
                .is_some_and(|declaration| declaration.name.as_str() == alias)
        }) || file
            .references()
            .iter()
            .any(|reference| reference.name.as_str() == alias)
        {
            continue;
        }
        return Ok(SelectedHelper {
            id,
            export,
            module,
            alias,
        });
    }
    Err(failure())
}
