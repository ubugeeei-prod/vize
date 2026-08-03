//! CSS-module metadata for per-SFC virtual TypeScript options.

use std::collections::{BTreeMap, BTreeSet};

use vize_atelier_sfc::SfcDescriptor;
use vize_carton::{String as CompactString, ToCompactString};

use crate::virtual_ts::{CSS_MODULE_GLOBAL_MARKER, TemplateGlobal, VirtualTsOptions};

pub(crate) fn virtual_ts_options_for_descriptor(
    base: &VirtualTsOptions,
    descriptor: &SfcDescriptor,
) -> VirtualTsOptions {
    // Per-file generation never re-emits the global auto-import stubs inline:
    // they are written once to a shared ambient `.d.ts`.
    let module_types = css_module_types(descriptor);
    let has_css_modules = !module_types.is_empty();
    let mut template_globals = base.template_globals.clone();
    let mut css_modules = Vec::new();
    for (module_name, classes) in module_types {
        template_globals.retain(|global| global.name != module_name);
        if let Some(classes) = classes {
            template_globals.push(TemplateGlobal {
                name: module_name,
                type_annotation: css_module_type_annotation(&classes),
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
        external_template_bindings: base.external_template_bindings.clone(),
        reference_paths: base.reference_paths.clone(),
    }
}

/// Collect the classes that are statically exported by each CSS module.
///
/// A module falls back to `Record<string, string>` when another file or CSS
/// Modules composition can contribute names. That deliberately trades typo
/// detection for soundness rather than inventing a closed shape.
fn css_module_types(
    descriptor: &SfcDescriptor<'_>,
) -> BTreeMap<CompactString, Option<BTreeSet<CompactString>>> {
    let mut modules = BTreeMap::new();
    for style in descriptor.styles.iter() {
        let Some(module_name) = style.module.as_ref() else {
            continue;
        };
        let module_name = module_name.to_compact_string();
        let is_plain_css = style
            .lang
            .as_deref()
            .is_none_or(|language| language.eq_ignore_ascii_case("css"));
        let authored_classes = (style.src.is_none()
            && is_plain_css
            && !contains_dynamic_css_module_exports(&style.content))
        .then(|| extract_authored_css_classes(&style.content))
        .flatten();

        let entry = modules
            .entry(module_name)
            .or_insert_with(|| Some(BTreeSet::new()));
        let Some(authored_classes) = authored_classes else {
            *entry = None;
            continue;
        };
        if let Some(existing) = entry.as_mut() {
            existing.extend(authored_classes);
        }
    }
    modules
}

fn contains_dynamic_css_module_exports(css: &str) -> bool {
    let lower = css.to_ascii_lowercase();
    ["@import", "@value", "composes", ":export", ":global", "#{"]
        .iter()
        .any(|marker| lower.contains(marker))
}

/// Extract simple authored class selectors from selector preludes. Declarations
/// are never scanned, so values such as `url(./asset.png)` cannot become fake
/// module keys. Comments and string contents are blanked before inspection.
fn extract_authored_css_classes(css: &str) -> Option<BTreeSet<CompactString>> {
    let bytes = css.as_bytes();
    let mut classes = BTreeSet::new();
    let mut prelude = Vec::new();
    let mut index = 0usize;
    let mut quote = None;

    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(open_quote) = quote {
            if byte == b'\\' {
                index = (index + 2).min(bytes.len());
                continue;
            }
            if byte == open_quote {
                quote = None;
            }
            index += 1;
            continue;
        }
        if byte == b'/' && bytes.get(index + 1) == Some(&b'*') {
            let relative_end = css[index + 2..].find("*/")?;
            index += relative_end + 4;
            continue;
        }
        if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
            index += 1;
            continue;
        }
        match byte {
            b'{' => {
                collect_classes_from_selector_prelude(&prelude, &mut classes)?;
                prelude.clear();
            }
            b';' | b'}' => prelude.clear(),
            _ => prelude.push(byte),
        }
        index += 1;
    }
    quote.is_none().then_some(classes)
}

fn collect_classes_from_selector_prelude(
    prelude: &[u8],
    classes: &mut BTreeSet<CompactString>,
) -> Option<()> {
    let trimmed = prelude
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .map_or(&[][..], |start| &prelude[start..]);
    if trimmed.starts_with(b"@") {
        return Some(());
    }
    if trimmed.contains(&b'\\') || !trimmed.is_ascii() {
        return None;
    }

    let mut index = 0usize;
    while index + 1 < trimmed.len() {
        if trimmed[index] != b'.' || !is_css_class_start(trimmed[index + 1]) {
            index += 1;
            continue;
        }
        let start = index + 1;
        let mut end = start + 1;
        while end < trimmed.len() && is_css_class_continue(trimmed[end]) {
            end += 1;
        }
        let class_name = std::str::from_utf8(&trimmed[start..end]).ok()?;
        classes.insert(class_name.to_compact_string());
        index = end;
    }
    Some(())
}

fn is_css_class_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'-')
}

fn is_css_class_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
}

fn css_module_type_annotation(classes: &BTreeSet<CompactString>) -> CompactString {
    let mut annotation = CompactString::from("{ ");
    for class_name in classes {
        annotation.push_str("readonly ");
        annotation.push_str(
            serde_json::to_string(class_name.as_str())
                .expect("CSS class name should serialize")
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
        contains_dynamic_css_module_exports, css_module_type_annotation,
        extract_authored_css_classes,
    };

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
            r#"{ readonly "nested-item": string; readonly "root": string; readonly "row": string; }"#
        );
    }

    #[test]
    fn rejects_selectors_that_cannot_form_a_closed_export_shape() {
        for source in [
            r#"@import "./base.css"; .root {}"#,
            r#".root { composes : shared from "./base.css"; }"#,
            r#":global(.external) .root {}"#,
            r#".#{dynamic} {}"#,
        ] {
            assert!(contains_dynamic_css_module_exports(source), "{source}");
        }
        assert!(extract_authored_css_classes(r#".café {}"#).is_none());
        assert!(extract_authored_css_classes(r#".escaped\:name {}"#).is_none());
    }
}
