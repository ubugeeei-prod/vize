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

fn merge_resolved(
    croquis: &mut Croquis,
    descriptor: &SfcDescriptor<'_>,
    filename: &str,
    sources: Option<&TypeSourceSnapshot>,
) {
    use crate::types::BindingType;
    let disk_sources = TypeSourceSnapshot::default();
    let world_sources = sources.unwrap_or(&disk_sources);

    let Some(script_setup) = descriptor.script_setup.as_ref() else {
        if let Some(script) = descriptor.script.as_ref() {
            let ctx = ScriptCompileContext::new(&script.content);
            croquis
                .types
                .set_resolved_world(ctx.resolve_type_world_with_sources(
                    filename,
                    None,
                    script.lang.as_deref() == Some("tsx"),
                    world_sources,
                ));
        }
        return;
    };

    // The scoped world reads the current setup/normal text directly. The
    // compatibility collector and context analysis do not contribute to it.
    let ctx = ScriptCompileContext::new(&script_setup.content);
    croquis.types.set_resolved_world(
        ctx.resolve_type_world_with_sources(
            filename,
            descriptor
                .script
                .as_ref()
                .map(|script| script.content.as_ref()),
            script_setup.lang.as_deref() == Some("tsx")
                || descriptor
                    .script
                    .as_ref()
                    .is_some_and(|script| script.lang.as_deref() == Some("tsx")),
            world_sources,
        ),
    );

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
