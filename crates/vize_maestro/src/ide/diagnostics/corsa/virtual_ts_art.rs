//! Virtual TypeScript generation for standalone `.art.vue` files.

use tower_lsp::lsp_types::Url;
use vize_canon::virtual_ts::{VirtualTsOptions, generate_virtual_ts_with_offsets};
use vize_croquis::{Drawer, DrawerOptions};

use super::super::{DiagnosticService, VirtualTsResult};
use super::virtual_ts::{collect_relative_vue_specifiers, rewrite_vue_imports};

fn quote_ts_string(value: &str) -> String {
    let mut quoted = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            _ => quoted.push(ch),
        }
    }
    quoted.push('"');
    quoted
}

fn to_safe_identifier_fragment(value: &str) -> String {
    let mut result = String::with_capacity(value.len().max(1));
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' {
            result.push(ch);
        } else {
            result.push('_');
        }
    }
    if result.is_empty() {
        result.push('_');
    }
    result
}

fn to_safe_identifier(value: &str) -> String {
    let mut result = to_safe_identifier_fragment(value);
    if !result
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_' || ch == '$')
    {
        result.insert(0, '_');
    }
    if is_reserved_identifier(result.as_str()) {
        result.insert(0, '_');
    }
    result
}

fn is_valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_' || first == '$')
        && !is_reserved_identifier(value)
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}

fn is_reserved_identifier(value: &str) -> bool {
    matches!(
        value,
        "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "null"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
    )
}

fn kebab_case_component_name(name: &str) -> Option<String> {
    let mut kebab = String::new();
    let mut previous_was_separator = false;

    for (index, ch) in name.char_indices() {
        if ch.is_ascii_uppercase() {
            if index > 0 && !previous_was_separator {
                kebab.push('-');
            }
            kebab.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if ch == '_' || ch == '-' || ch.is_whitespace() {
            if !kebab.ends_with('-') && !kebab.is_empty() {
                kebab.push('-');
            }
            previous_was_separator = true;
        } else {
            kebab.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        }
    }

    while kebab.ends_with('-') {
        kebab.pop();
    }
    (kebab.contains('-') && kebab != name).then_some(kebab)
}

fn add_art_target_component_bindings(
    options: &mut VirtualTsOptions,
    summary: &vize_croquis::Croquis,
    target: &crate::virtual_code::ArtTargetComponent,
) {
    if target.source.trim().is_empty() {
        return;
    }

    let has_component_binding = summary.bindings.bindings.contains_key(target.name.as_str());
    let mut component_ref = target.name.clone();

    if !has_component_binding {
        let import_alias = vize_carton::cstr!(
            "__VizeArtTarget_{}",
            to_safe_identifier_fragment(target.name.as_str())
        );
        options.auto_import_stubs.push(vize_carton::cstr!(
            "import {import_alias} from {};",
            quote_ts_string(target.source.as_str())
        ));
        component_ref = import_alias.to_string();

        if is_valid_identifier(target.name.as_str()) {
            options.auto_import_stubs.push(vize_carton::cstr!(
                "const {} = {import_alias};",
                target.name
            ));
            options
                .external_template_bindings
                .push(target.name.clone().into());
            component_ref = target.name.clone();
        }
    }

    if has_component_binding {
        options
            .external_template_bindings
            .push(target.name.clone().into());
    }

    if let Some(kebab_name) = kebab_case_component_name(target.name.as_str()) {
        let kebab_ref = to_safe_identifier(kebab_name.as_str());
        options
            .auto_import_stubs
            .push(vize_carton::cstr!("const {kebab_ref} = {component_ref};"));
        options.external_template_bindings.push(kebab_name.into());
    }
}

impl DiagnosticService {
    pub(in crate::ide::diagnostics) fn generate_virtual_ts_for_art(
        uri: &Url,
        content: &str,
    ) -> Option<VirtualTsResult> {
        let art_allocator = vize_carton::Bump::new();
        let art_desc = vize_musea::parse_art(
            &art_allocator,
            content,
            vize_musea::ArtParseOptions::default(),
        )
        .ok()?;

        let (_, variant) = art_desc
            .variants
            .iter()
            .enumerate()
            .find(|(_, variant)| variant.is_default)
            .or_else(|| art_desc.variants.iter().enumerate().next())?;
        let template_content = variant.template;
        if template_content.trim().is_empty() {
            return None;
        }

        let template_ptr = template_content.as_ptr() as usize;
        let source_ptr = content.as_ptr() as usize;
        let template_offset = (template_ptr - source_ptr) as u32;

        let descriptor = vize_atelier_sfc::parse_sfc(
            content,
            vize_atelier_sfc::SfcParseOptions {
                filename: uri.path().to_string().into(),
                ..Default::default()
            },
        )
        .ok()?;

        let target_component = descriptor
            .script_setup
            .as_ref()
            .and_then(|script_setup| {
                crate::virtual_code::find_define_art_target_component(script_setup.content.as_ref())
            })
            .or_else(|| {
                art_desc
                    .metadata
                    .component
                    .and_then(crate::virtual_code::art_target_component_from_source)
            });

        let mut combined_script = String::new();
        let (script_offset, sfc_script_start_line) =
            if let Some(script_setup) = descriptor.script_setup.as_ref() {
                let isolate = !script_setup
                    .attrs
                    .get("isolate")
                    .is_some_and(|value| value.as_ref().eq_ignore_ascii_case("false"));
                let parts = crate::virtual_code::analyze_art_script_setup(
                    script_setup.content.as_ref(),
                    script_setup.loc.start,
                    isolate,
                );

                for chunk in parts
                    .shared_imports
                    .iter()
                    .chain(parts.isolated_body.iter())
                {
                    combined_script.push_str(&chunk.text);
                    if !combined_script.ends_with('\n') {
                        combined_script.push('\n');
                    }
                }

                (
                    script_setup.loc.start as u32,
                    script_setup.loc.start_line as u32,
                )
            } else if let Some(script) = descriptor.script.as_ref() {
                combined_script.push_str(script.content.as_ref());
                if !combined_script.ends_with('\n') {
                    combined_script.push('\n');
                }
                (script.loc.start as u32, script.loc.start_line as u32)
            } else {
                (0, 1)
            };

        let script_content = combined_script.as_str();
        let template_allocator = vize_carton::Bump::new();
        let (template_ast, _) = vize_armature::parse(&template_allocator, template_content);

        let mut analyzer = Drawer::with_options(DrawerOptions::full());
        analyzer.analyze_script(script_content);
        analyzer.analyze_template(&template_ast);

        let summary = analyzer.finish();
        let mut virtual_ts_options = VirtualTsOptions::default();
        if let Some(target) = target_component.as_ref() {
            add_art_target_component_bindings(&mut virtual_ts_options, &summary, target);
        }

        let output = generate_virtual_ts_with_offsets(
            &summary,
            Some(script_content),
            Some(&template_ast),
            script_offset,
            template_offset,
            &virtual_ts_options,
        );
        let code = output.code;
        let line_mappings = Self::parse_vize_map_comments(&code);
        let relative_vue_imports = collect_relative_vue_specifiers(&code);
        let (rewritten_code, import_source_map) = rewrite_vue_imports(&code);

        Some(VirtualTsResult {
            code: rewritten_code,
            source_mappings: output.mappings,
            import_source_map,
            relative_vue_imports,
            user_code_start_line: code
                .lines()
                .enumerate()
                .find(|(_, line)| line.contains("// User setup code"))
                .map(|(i, _)| i as u32 + 1)
                .unwrap_or(0),
            sfc_script_start_line,
            template_scope_start_line: code
                .lines()
                .enumerate()
                .find(|(_, line)| line.contains("Template Scope"))
                .map(|(i, _)| i as u32)
                .unwrap_or(u32::MAX),
            line_mappings,
            skipped_import_lines: Self::count_import_lines(script_content),
        })
    }
}
