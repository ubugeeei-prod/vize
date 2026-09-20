//! CSS-module metadata for per-SFC virtual TypeScript options.

use std::collections::{BTreeMap, BTreeSet};

use vize_atelier_sfc::SfcDescriptor;
use vize_carton::{String as CompactString, ToCompactString};

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
                    .expect("import specifier should serialize")
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
    ["@value", "composes", ":export", ":global", "#{"]
        .iter()
        .any(|marker| lower.contains(marker))
}

/// The specifiers of the block's `@import` rules, in source order: plain
/// `@import "x"` and `@import url("x")`, the forms Vue Language Tools reads.
fn css_import_targets(css: &str) -> Vec<CompactString> {
    let mut targets = Vec::new();
    let mut rest = css;
    while let Some(index) = rest.find("@import") {
        rest = rest[index + "@import".len()..].trim_start();
        let inner = rest
            .strip_prefix("url(")
            .map(|after| after.trim_start())
            .unwrap_or(rest);
        let Some(quote) = inner.chars().next().filter(|c| matches!(c, '"' | '\'')) else {
            continue;
        };
        let Some(end) = inner[1..].find(quote) else {
            break;
        };
        targets.push(inner[1..1 + end].to_compact_string());
        rest = &inner[1 + end + 1..];
    }
    targets
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
mod tests;
