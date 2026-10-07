//! Single-file props shapes that need a cross-file provider.

use vize_croquis::{Croquis, types::TypeDefinitions};
use vize_l0::{CompactString, FxHashSet};

pub(super) fn has_unresolved_props(analysis: &Croquis) -> bool {
    let Some(type_args) = analysis
        .macros
        .define_props()
        .and_then(|call| call.type_args.as_deref())
    else {
        return false;
    };
    if let Some(complete) = analysis.types.resolved_props_complete() {
        return !complete;
    }
    let type_args = type_args
        .strip_prefix('<')
        .and_then(|inner| inner.strip_suffix('>'))
        .unwrap_or(type_args);
    imported_shape(
        analysis.types.definitions(),
        type_args,
        &mut FxHashSet::default(),
    )
}

fn imported_shape(
    definitions: &TypeDefinitions,
    shape: &str,
    visiting: &mut FxHashSet<CompactString>,
) -> bool {
    let shape = shape.trim();
    // Imported member value types cannot contribute additional prop names.
    if shape.starts_with('{') {
        return false;
    }
    let (name, arguments) = shape
        .split_once('<')
        .map_or((shape, None), |(name, args)| (name.trim(), Some(args)));
    if definitions.is_imported(name) {
        return true;
    }
    if matches!(name, "Partial" | "Required" | "Readonly" | "Pick" | "Omit") {
        return arguments.is_some_and(|arguments| {
            let base = arguments
                .trim_end_matches('>')
                .split(',')
                .next()
                .unwrap_or_default();
            imported_shape(definitions, base, visiting)
        });
    }
    if shape.contains(['&', '|']) {
        return shape
            .split(['&', '|'])
            .any(|part| imported_shape(definitions, part, visiting));
    }
    if !visiting.insert(CompactString::new(name)) {
        return false;
    }
    let imported = definitions
        .type_aliases
        .get(name)
        .is_some_and(|body| imported_shape(definitions, body, visiting))
        || definitions
            .interface_extends(name)
            .iter()
            .any(|base| imported_shape(definitions, base, visiting));
    visiting.remove(name);
    imported
}
