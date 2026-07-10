use vize_canon::virtual_ts::{TemplateGlobal, VirtualTsOptions};
use vize_carton::ToCompactString;

pub(super) fn collect(options: &mut VirtualTsOptions) {
    for (name, type_annotation) in [
        ("$config", "any"),
        (
            "$fetchState",
            "{ pending: boolean; error: any; timestamp: number; [key: string]: any }",
        ),
        ("$nuxt", "any"),
        ("$route", "any"),
        ("$router", "any"),
        ("$store", "any"),
    ] {
        if options
            .template_globals
            .iter()
            .any(|global| global.name == name)
        {
            continue;
        }
        options.template_globals.push(TemplateGlobal {
            name: name.to_compact_string(),
            type_annotation: type_annotation.to_compact_string(),
            default_value: "undefined as any".into(),
        });
    }
}
