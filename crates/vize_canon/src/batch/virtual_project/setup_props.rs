use std::path::Path;

use vize_atelier_sfc::{SfcDescriptor, croquis::merge_resolved_props_into_croquis_with_sources};
use vize_carton::FxHashSet;
use vize_croquis::{BindingType, Croquis, macros::PropDefinition};

mod cache;
mod default_props;
mod imports;
mod runtime_props;
mod syntax;

pub(super) use cache::RuntimePropResolveCache;

use crate::virtual_ts::RESOLVED_RUNTIME_DEFAULT;
use default_props::merge_type_based_with_defaults_into_croquis;
use imports::find_imported_runtime_binding;
use runtime_props::{resolve_imported_runtime_props, resolve_inline_runtime_props};
use syntax::runtime_arg_identifier;

pub(super) fn augment_type_based_props_from_script_context(
    croquis: &mut Croquis,
    descriptor: &SfcDescriptor<'_>,
    path: &Path,
    cache: &RuntimePropResolveCache,
) {
    let path_string = path.to_string_lossy();
    merge_resolved_props_into_croquis_with_sources(
        croquis,
        descriptor,
        path_string.as_ref(),
        &cache.type_sources,
    );
    merge_imported_runtime_props_into_croquis(croquis, descriptor, path, cache);

    let script_setup_source = descriptor
        .script_setup
        .as_ref()
        .map(|script| script.content.as_ref());
    if let Some(script_setup_source) = script_setup_source {
        merge_type_based_with_defaults_into_croquis(
            croquis,
            script_setup_source,
            descriptor
                .script
                .as_ref()
                .map(|script| script.content.as_ref()),
            path,
            cache,
        );
    }
}

fn merge_imported_runtime_props_into_croquis(
    croquis: &mut Croquis,
    descriptor: &SfcDescriptor<'_>,
    path: &Path,
    cache: &RuntimePropResolveCache,
) {
    let Some(script_setup) = descriptor.script_setup.as_ref() else {
        return;
    };
    let Some(runtime_arg) = croquis
        .macros
        .define_props()
        .filter(|call| call.type_args.is_none())
        .and_then(|call| call.runtime_args.as_deref())
    else {
        return;
    };

    let mut visited = FxHashSet::default();
    let props = if let Some(runtime_binding) = runtime_arg_identifier(runtime_arg) {
        let import =
            find_imported_runtime_binding(&script_setup.content, runtime_binding).or_else(|| {
                descriptor.script.as_ref().and_then(|script| {
                    find_imported_runtime_binding(&script.content, runtime_binding)
                })
            });
        let Some(import) = import else {
            return;
        };
        resolve_imported_runtime_props(
            path,
            import.source.as_str(),
            import.imported.as_str(),
            runtime_binding,
            &mut visited,
            cache,
        )
    } else {
        resolve_inline_runtime_props(&script_setup.content, path, &mut visited, cache)
    };
    if props.is_empty() {
        return;
    }

    merge_resolved_runtime_props_into_croquis(croquis, props);
}

fn merge_resolved_runtime_props_into_croquis(croquis: &mut Croquis, props: Vec<PropDefinition>) {
    croquis.macros.set_resolved_prop_defaults(
        props
            .iter()
            .filter(|prop| prop.default_value.is_some())
            .map(|prop| prop.name.clone())
            .collect(),
    );

    let mut known: FxHashSet<_> = croquis
        .macros
        .props()
        .iter()
        .map(|prop| prop.name.clone())
        .collect();
    for mut prop in props {
        if known.contains(&prop.name) {
            if let Some(prop_type) = prop.prop_type.as_deref() {
                croquis
                    .macros
                    .refine_unknown_prop_type(prop.name.as_str(), prop_type);
            }
            continue;
        }
        known.insert(prop.name.clone());
        if !croquis.has_binding(prop.name.as_str()) {
            croquis.note_binding(prop.name.as_str(), BindingType::Props);
        }
        if prop.default_value.as_deref() == Some(RESOLVED_RUNTIME_DEFAULT) {
            prop.default_value = None;
        }
        croquis.macros.add_prop(prop);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_runtime_defaults_keep_presence_separate_from_authored_metadata() {
        let mut croquis = Croquis::default();
        merge_resolved_runtime_props_into_croquis(
            &mut croquis,
            vec![
                PropDefinition {
                    name: "resolved".into(),
                    prop_type: Some("string".into()),
                    required: false,
                    default_value: Some(RESOLVED_RUNTIME_DEFAULT.into()),
                },
                PropDefinition {
                    name: "authored".into(),
                    prop_type: Some("number".into()),
                    required: true,
                    default_value: Some("42".into()),
                },
            ],
        );

        let mut resolved = croquis.macros.resolved_prop_defaults().collect::<Vec<_>>();
        resolved.sort_unstable();
        assert_eq!(resolved, ["authored", "resolved"]);
        assert_eq!(
            croquis
                .macros
                .props()
                .iter()
                .map(|prop| (
                    prop.name.as_str(),
                    prop.prop_type.as_deref(),
                    prop.required,
                    prop.default_value.as_deref(),
                ))
                .collect::<Vec<_>>(),
            [
                ("resolved", Some("string"), false, None),
                ("authored", Some("number"), true, Some("42")),
            ]
        );
    }
}
