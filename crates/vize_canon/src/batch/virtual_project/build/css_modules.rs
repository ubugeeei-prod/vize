//! CSS-module metadata for per-SFC virtual TypeScript options.

use std::collections::{BTreeMap, BTreeSet};

use vize_atelier_sfc::SfcDescriptor;
use vize_carton::{String as CompactString, ToCompactString};

use crate::css_classes::authored_local_css_classes as extract_authored_css_classes;
use crate::virtual_ts::{
    CSS_MODULE_GLOBAL_MARKER, ResolveStyleClassNames, TemplateGlobal, VirtualTsOptions,
};

pub(crate) fn virtual_ts_options_for_descriptor(
    base: &VirtualTsOptions,
    descriptor: &SfcDescriptor,
    strict: bool,
    resolve_style_imports: bool,
) -> VirtualTsOptions {
    // Per-file generation never re-emits the global auto-import stubs inline:
    // they are written once to a shared ambient `.d.ts`.
    let module_types = css_module_types(descriptor, resolve_style_imports);
    let has_css_modules = !module_types.is_empty();
    let mut template_globals = base.template_globals.clone();
    let mut css_modules = Vec::new();
    for (module_name, shape) in module_types {
        template_globals.retain(|global| global.name != module_name);
        if let Some(shape) = shape {
            template_globals.push(TemplateGlobal {
                name: module_name,
                type_annotation: css_module_type(&shape, strict),
                default_value: CSS_MODULE_GLOBAL_MARKER.into(),
            });
        } else {
            css_modules.push(module_name);
        }
    }
    let css_modules = if !has_css_modules {
        base.css_modules.clone()
    } else {
        css_modules
    };

    VirtualTsOptions {
        template_globals,
        css_modules,
        auto_import_stubs: Vec::new(),
        // The stubs move to the shared ambient `.d.ts`, but their names must
        // stay: template scope unwraps auto-imported refs the same way it
        // unwraps an authored `<script setup>` import (#4146).
        auto_import_bindings: base.auto_import_binding_names(),
        external_template_bindings: base.external_template_bindings.clone(),
        reference_paths: base.reference_paths.clone(),
        strict_instance_globals: base.strict_instance_globals,
    }
}

/// Class names the SFC's `<style>` blocks declare, deduplicated in source
/// order, for the `__VLS_StyleScopedClasses` type: every block under
/// `resolveStyleClassNames: true`, the scoped ones by default. A block whose
/// selectors cannot be read statically contributes nothing.
pub(crate) fn style_scoped_class_names(
    descriptor: &SfcDescriptor<'_>,
    mode: ResolveStyleClassNames,
) -> Vec<CompactString> {
    let mut names = Vec::new();
    for style in descriptor.styles.iter() {
        let included = match mode {
            ResolveStyleClassNames::None => false,
            ResolveStyleClassNames::Scoped => style.scoped,
            ResolveStyleClassNames::All => true,
        };
        if !included || style.src.is_some() {
            continue;
        }
        for class_name in extract_authored_css_classes(&style.content).unwrap_or_default() {
            if !names.contains(&class_name) {
                names.push(class_name);
            }
        }
    }
    names
}

/// The statically known shape of one CSS module: its authored class names
/// and, under `resolveStyleImports`, the stylesheets whose default export it
/// also carries (`src` first, then `@import` targets in source order).
#[derive(Default)]
pub(crate) struct CssModuleShape {
    classes: BTreeSet<CompactString>,
    imports: Vec<CompactString>,
}

/// Collect the classes that are statically exported by each CSS module.
///
/// A module falls back to `Record<string, string>` when another file or CSS
/// Modules composition can contribute names. That deliberately trades typo
/// detection for soundness rather than inventing a closed shape.
fn css_module_types(
    descriptor: &SfcDescriptor<'_>,
    resolve_style_imports: bool,
) -> BTreeMap<CompactString, Option<CssModuleShape>> {
    let mut modules = BTreeMap::new();
    for style in descriptor.styles.iter() {
        let Some(module_name) = style.module.as_ref() else {
            continue;
        };
        let module_name = module_name.to_compact_string();
        let shape = css_module_block_shape(style, resolve_style_imports);

        let entry = modules
            .entry(module_name)
            .or_insert_with(|| Some(CssModuleShape::default()));
        let Some(shape) = shape else {
            *entry = None;
            continue;
        };
        if let Some(existing) = entry.as_mut() {
            existing.classes.extend(shape.classes);
            existing.imports.extend(shape.imports);
        }
    }
    modules
}

/// One `<style module>` block's shape, or `None` when its names cannot be
/// known statically (a preprocessor, CSS Modules composition, or an imported
/// stylesheet the project has not asked to resolve).
fn css_module_block_shape(
    style: &vize_atelier_sfc::SfcStyleBlock<'_>,
    resolve_style_imports: bool,
) -> Option<CssModuleShape> {
    let is_plain_css = style
        .lang
        .as_deref()
        .is_none_or(|language| language.eq_ignore_ascii_case("css"));
    if !is_plain_css || contains_dynamic_css_module_exports(&style.content) {
        return None;
    }
    let mut imports = Vec::new();
    if let Some(src) = style.src.as_deref() {
        if !resolve_style_imports {
            return None;
        }
        imports.push(src.to_compact_string());
    }
    let css_imports = css_import_targets(&style.content);
    if !css_imports.is_empty() {
        if !resolve_style_imports {
            return None;
        }
        imports.extend(css_imports);
    }
    Some(CssModuleShape {
        classes: extract_authored_css_classes(&style.content)?,
        imports,
    })
}

/// The type one `<style module>` block declares on its own, for the
/// duplicate-name literal: the same text for the same classes, so a repeated
/// name with equal classes is only a duplicate identifier while different
/// classes also change the declared type, as `vue-tsc` reports.
pub(crate) fn css_module_block_type(
    style: &vize_atelier_sfc::SfcStyleBlock<'_>,
    strict: bool,
) -> CompactString {
    match css_module_block_shape(style, false) {
        Some(shape) => css_module_type(&shape, strict),
        None => CompactString::from("Record<string, string>"),
    }
}

fn css_module_type(shape: &CssModuleShape, strict: bool) -> CompactString {
    let classes = css_module_type_annotation(&shape.classes);
    // Imported stylesheets contribute their default export; the whole
    // intersection is flattened the way `vue-tsc` flattens it, so the type
    // is one object literal a caller can compare exactly.
    let own = if shape.imports.is_empty() {
        classes
    } else {
        let mut merged = CompactString::from("__VizePrettify<{}");
        for import in &shape.imports {
            merged.push_str(" & typeof import(");
            merged.push_str(
                serde_json::to_string(import.as_str())
                    .unwrap_or_default()
                    .as_str(),
            );
            merged.push_str(").default");
        }
        if !shape.classes.is_empty() {
            merged.push_str(" & ");
            merged.push_str(classes.as_str());
        }
        merged.push('>');
        merged
    };
    if strict {
        own
    } else {
        vize_carton::cstr!("Record<string, string> & {own}")
    }
}

fn contains_dynamic_css_module_exports(css: &str) -> bool {
    let lower = css.to_ascii_lowercase();
    ["@value", "composes", ":export", "#{"]
        .iter()
        .any(|marker| lower.contains(marker))
}

/// The specifiers of the block's `@import` rules, in source order: plain
/// `@import "x"` and `@import url("x")`, the forms Vue Language Tools reads.
fn css_import_targets(css: &str) -> Vec<CompactString> {
    let mut targets = Vec::new();
    let mut rest = css;
    while let Some((_, after_import)) = rest.split_once("@import") {
        rest = after_import.trim_start();
        let inner = rest
            .strip_prefix("url(")
            .map(|after| after.trim_start())
            .unwrap_or(rest);
        let Some(quote) = inner.chars().next().filter(|c| matches!(c, '"' | '\'')) else {
            continue;
        };
        let Some((target, after_target)) = inner.get(1..).and_then(|body| body.split_once(quote))
        else {
            break;
        };
        targets.push(target.to_compact_string());
        rest = after_target;
    }
    targets
}

fn css_module_type_annotation(classes: &BTreeSet<CompactString>) -> CompactString {
    let mut annotation = CompactString::from("{ ");
    for class_name in classes {
        annotation.push_str(
            serde_json::to_string(class_name.as_str())
                .unwrap_or_default()
                .as_str(),
        );
        annotation.push_str(": string; ");
    }
    annotation.push('}');
    annotation
}

#[cfg(test)]
mod tests {
    use super::{
        contains_dynamic_css_module_exports, css_import_targets, css_module_block_shape,
        css_module_block_type, css_module_type, css_module_type_annotation,
        extract_authored_css_classes,
    };

    #[test]
    fn import_targets_follow_source_order_in_both_forms() {
        assert_eq!(
            css_import_targets(
                "@import \"./a.css\";\n@import url('./b.css');\n@import url(./c.css);\n.x { }"
            ),
            vec!["./a.css", "./b.css"]
        );
        assert!(css_import_targets(".a { }").is_empty());
    }

    #[test]
    fn extracts_only_classes_from_selector_preludes() {
        let classes = extract_authored_css_classes(
            r#"
/* .commented-out {} */
.root, .row:hover {
  background: url(./asset.png);
  content: ".not-a-class";
}
@media (width > 20rem) {
  .nested-item { color: green; }
}
"#,
        )
        .expect("plain CSS selectors should resolve");
        assert_eq!(
            classes.iter().map(|name| name.as_str()).collect::<Vec<_>>(),
            ["nested-item", "root", "row"]
        );
        assert_eq!(
            css_module_type_annotation(&classes).as_str(),
            r#"{ "nested-item": string; "root": string; "row": string; }"#
        );
    }

    #[test]
    fn rejects_selectors_that_cannot_form_a_closed_export_shape() {
        for source in [
            r#".root { composes : shared from "./base.css"; }"#,
            r#".#{dynamic} {}"#,
        ] {
            assert!(contains_dynamic_css_module_exports(source), "{source}");
        }
        assert!(extract_authored_css_classes(r#".café {}"#).is_none());
        assert!(extract_authored_css_classes(r#".escaped\:name {}"#).is_none());
    }

    // `@import` names another stylesheet's classes: unknown without
    // `resolveStyleImports`, the imported default export's type with it.
    #[test]
    fn imported_stylesheets_close_the_shape_only_when_resolved() {
        let source = "<style module>\n@import \"./base.css\";\n.root {}\n</style>\n";
        let descriptor =
            vize_atelier_sfc::parse_sfc(source, Default::default()).expect("SFC parses");
        let style = &descriptor.styles[0];

        assert!(css_module_block_shape(style, false).is_none());
        let shape = css_module_block_shape(style, true).expect("imports are resolved");
        assert_eq!(
            css_module_type(&shape, false),
            "Record<string, string> & __VizePrettify<{} & typeof import(\"./base.css\").default & { \"root\": string; }>"
        );
        assert_eq!(
            css_module_type(&shape, true),
            "__VizePrettify<{} & typeof import(\"./base.css\").default & { \"root\": string; }>"
        );
        assert_eq!(
            css_module_block_type(style, false),
            "Record<string, string>",
            "the duplicate-name literal never resolves imports"
        );
    }

    // Global selectors do not export names, but cannot erase adjacent local keys
    // (#9378). A known key must stay `string` with noUncheckedIndexedAccess.
    #[test]
    fn global_and_local_scopes_preserve_only_authored_module_exports() {
        for (css, expected) in [
            (".active:global(.loaded) {}", vec!["active"]),
            (":global(.external) .root {}", vec!["root"]),
            (":global .external .another, .root {}", vec!["root"]),
            (":global .external :local .root {}", vec!["root"]),
            (":global(.external) :local(.root) {}", vec!["root"]),
            (
                ":local(.root, .row):global(.loaded) {}",
                vec!["root", "row"],
            ),
            (".root:is(:global(.external), .row) {}", vec!["root", "row"]),
            // PostCSS CSS Modules keeps bare global mode across the function's
            // selector list, unlike the narrow functional :global(...) above.
            (".bar:not(:global .foo, .baz) {}", vec!["bar"]),
            (".root:is(:global .external, .row) {}", vec!["root"]),
            (
                ".root:is(:global .external, .row) .after {}",
                vec!["after", "root"],
            ),
            (
                ".root:is(:global .external, .row), .after {}",
                vec!["after", "root"],
            ),
            (
                ".root:is(:global .external, :local .row) .after {}",
                vec!["after", "root", "row"],
            ),
            ("[data-kind=.attribute] .root {}", vec!["root"]),
            (":global(.external) {}", vec![]),
        ] {
            let classes = extract_authored_css_classes(css).expect("static selector");
            assert_eq!(
                classes.iter().map(|name| name.as_str()).collect::<Vec<_>>(),
                expected,
                "{css}"
            );
        }
        for css in [":global(.external {}", ".root) {}", "[data-kind=.x {}"] {
            assert!(extract_authored_css_classes(css).is_none(), "{css}");
        }
    }

    #[test]
    fn local_classes_in_global_selectors_keep_the_strict_module_shape() {
        let source = "<style module>.active:global(.loaded) { color: red; }</style>";
        let descriptor =
            vize_atelier_sfc::parse_sfc(source, Default::default()).expect("SFC parses");
        let shape =
            css_module_block_shape(&descriptor.styles[0], false).expect("known local exports");
        assert_eq!(css_module_type(&shape, true), r#"{ "active": string; }"#);
        assert_eq!(
            css_module_type(&shape, false),
            r#"Record<string, string> & { "active": string; }"#
        );
    }

    #[test]
    fn composed_exports_still_need_the_existing_dynamic_fallback() {
        let source = r#"<style module>.active:global(.loaded) { composes: other from "./base.css"; }</style>"#;
        let descriptor =
            vize_atelier_sfc::parse_sfc(source, Default::default()).expect("SFC parses");
        assert!(css_module_block_shape(&descriptor.styles[0], false).is_none());
        assert_eq!(
            css_module_block_type(&descriptor.styles[0], true),
            "Record<string, string>"
        );
    }
}
