use crate::script::{ScriptCompileContext, TypeSourceSnapshot};
use crate::types::SfcDescriptor;
use vize_carton::FxHashSet;
use vize_croquis::Croquis;

/// Merge props resolved by the script compile context — which performs
/// cross-file and node_modules type resolution — into a Croquis summary.
///
/// Croquis alone cannot resolve props inherited through imported or heritage
/// types (`interface Props extends Omit<ImportedProps, ...>`), so
/// template-binding checks and virtual TS generation would treat those props
/// as undefined references. This mirrors the merge the compiler performs in
/// `compile.rs`.
pub fn merge_resolved_props_into_croquis(
    croquis: &mut Croquis,
    descriptor: &SfcDescriptor<'_>,
    filename: &str,
) {
    merge_resolved(croquis, descriptor, filename, None);
}

pub fn merge_resolved_props_into_croquis_with_sources(
    croquis: &mut Croquis,
    descriptor: &SfcDescriptor<'_>,
    filename: &str,
    sources: &TypeSourceSnapshot,
) {
    merge_resolved(croquis, descriptor, filename, Some(sources));
}

/// The scoped type world of a descriptor's scripts, read from disk, without
/// building a Croquis summary.
pub fn resolve_descriptor_type_world(
    descriptor: &SfcDescriptor<'_>,
    filename: &str,
) -> Option<vize_croquis::types::ResolvedTypeWorld> {
    descriptor_world(descriptor, filename, &TypeSourceSnapshot::default())
}

// The scoped world reads the current setup/normal text directly. The
// compatibility collector and context analysis do not contribute to it.
fn descriptor_world(
    descriptor: &SfcDescriptor<'_>,
    filename: &str,
    sources: &TypeSourceSnapshot,
) -> Option<vize_croquis::types::ResolvedTypeWorld> {
    let tsx = |block: Option<&crate::types::SfcScriptBlock<'_>>| {
        block.is_some_and(|block| block.lang.as_deref() == Some("tsx"))
    };
    let Some(script_setup) = descriptor.script_setup.as_ref() else {
        let script = descriptor.script.as_ref()?;
        return Some(
            ScriptCompileContext::new(&script.content).resolve_type_world_with_sources(
                filename,
                None,
                tsx(Some(script)),
                sources,
            ),
        );
    };
    Some(
        ScriptCompileContext::new(&script_setup.content).resolve_type_world_with_sources(
            filename,
            descriptor
                .script
                .as_ref()
                .map(|script| script.content.as_ref()),
            tsx(Some(script_setup)) || tsx(descriptor.script.as_ref()),
            sources,
        ),
    )
}

fn merge_resolved(
    croquis: &mut Croquis,
    descriptor: &SfcDescriptor<'_>,
    filename: &str,
    sources: Option<&TypeSourceSnapshot>,
) {
    use crate::types::BindingType;
    let disk_sources = TypeSourceSnapshot::default();
    let world_sources = sources.unwrap_or(&disk_sources);

    if let Some(world) = descriptor_world(descriptor, filename, world_sources) {
        croquis.types.set_resolved_world(world);
    }
    if descriptor.script_setup.is_none() {
        return;
    }

    let Some(type_args) = croquis
        .macros
        .define_props()
        .and_then(|call| call.type_args.as_deref())
    else {
        return;
    };
    let type_args = type_args
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
        .unwrap_or(type_args);
    let mut known: FxHashSet<_> = croquis
        .macros
        .props()
        .iter()
        .map(|prop| prop.name.clone())
        .collect();
    let Some(resolved) = croquis
        .types
        .resolved_world()
        .map(|world| world.resolve_properties(type_args))
    else {
        return;
    };
    croquis.types.record_resolved_properties(&resolved);
    for prop in resolved.properties {
        if !known.insert(prop.name.clone()) {
            continue;
        }
        if !croquis.bindings.contains(prop.name.as_str()) {
            croquis.bindings.add(prop.name.as_str(), BindingType::Props);
        }
        croquis
            .macros
            .add_prop(vize_croquis::macros::PropDefinition {
                name: prop.name,
                prop_type: prop.prop_type,
                required: !prop.optional,
                default_value: None,
            });
    }
}
