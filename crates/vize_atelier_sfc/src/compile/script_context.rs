//! Assemble macro type information through one selected project authority.

use crate::script::ScriptCompileContext;
use crate::types::{SfcDescriptor, SfcError};
use vize_l0::String;

use super::is_ts_lang;

pub(super) struct CollectedContext {
    pub context: ScriptCompileContext,
    pub imports: Vec<(String, String)>,
    pub dependencies: Vec<String>,
}

pub(super) fn collect(
    descriptor: &SfcDescriptor<'_>,
    source: &str,
    filename: &str,
    project: Option<&crate::script::SfcTypeProject>,
) -> Result<CollectedContext, SfcError> {
    let mut context = ScriptCompileContext::new(source);
    if let Some(script) = descriptor.script.as_ref() {
        context.collect_types_from(&script.content);
    }
    let error = |message: String| SfcError {
        message,
        code: Some("V_SFC_TYPE_RESOLUTION".into()),
        loc: descriptor
            .script_setup
            .as_ref()
            .map(|script| script.loc.clone()),
    };
    let mut imports = Vec::new();
    let mut dependencies = Vec::new();
    let mut declarations = Vec::new();
    let mut bindings = Vec::new();
    if let Some(script) = descriptor.script_setup.as_ref() {
        let collected = context
            .try_collect_imported_types_from_path(
                source,
                filename,
                is_ts_lang(script.lang.as_deref()),
                project,
            )
            .map_err(error)?;
        imports.extend(collected.imports);
        dependencies.extend(collected.dependencies);
        declarations.extend(collected.declarations);
        bindings.extend(collected.bindings);
    }
    if let Some(script) = descriptor.script.as_ref() {
        let collected = context
            .try_collect_imported_types_from_path(
                &script.content,
                filename,
                is_ts_lang(script.lang.as_deref()),
                project,
            )
            .map_err(error)?;
        imports.extend(collected.imports);
        // Each script's names are sorted and distinct already.
        if dependencies.is_empty() {
            dependencies = collected.dependencies;
        } else if !collected.dependencies.is_empty() {
            dependencies.extend(collected.dependencies);
            dependencies.sort();
            dependencies.dedup();
        }
        declarations.extend(collected.declarations);
        bindings.extend(collected.bindings);
    }
    // Every type text this component resolves starts in one of these.
    let mut texts = vec![source];
    for script in [descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
        .into_iter()
        .flatten()
    {
        texts.push(&script.content);
        texts.extend(script.attrs.values().map(AsRef::as_ref));
    }
    context.merge_referenced_declarations(&bindings, &declarations, &texts);
    Ok(CollectedContext {
        context,
        imports,
        dependencies,
    })
}

pub(super) fn validate(
    context: &ScriptCompileContext,
    imports: &[(String, String)],
    descriptor: &SfcDescriptor<'_>,
) -> Result<(), SfcError> {
    context
        .validate_macro_types(imports)
        .map_err(|message| SfcError {
            message,
            code: Some("V_SFC_TYPE_RESOLUTION".into()),
            loc: descriptor
                .script_setup
                .as_ref()
                .map(|script| script.loc.clone()),
        })
}
