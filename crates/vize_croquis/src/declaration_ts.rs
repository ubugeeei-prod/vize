//! Declaration TypeScript generation from Croquis analysis.
//!
//! This module turns the semantic sketch that Croquis already collected from an
//! SFC script into a lightweight `.d.ts` surface for Vue component consumers.

use crate::{Croquis, ScopeData, ScopeKind};
use vize_carton::{append, cstr, SmallVec, String};

/// Result of declaration generation.
#[derive(Debug, Clone, Default)]
pub struct DeclarationTsOutput {
    /// Generated `.d.ts` content.
    pub content: String,
}

/// Generate component declarations from Croquis analysis and the analyzed
/// script content.
pub fn generate_declaration_ts(
    summary: &Croquis,
    script_content: Option<&str>,
) -> DeclarationTsOutput {
    generate_declaration_ts_inner(summary, script_content.map(ScriptContent::Single))
}

/// Generate component declarations from Croquis analysis when plain `<script>`
/// and `<script setup>` were analyzed as a virtual concatenation.
///
/// This avoids allocating that concatenated script just to slice module-level
/// statements back out during declaration generation.
pub fn generate_declaration_ts_with_split_scripts(
    summary: &Croquis,
    plain_script: &str,
    setup_script: &str,
) -> DeclarationTsOutput {
    generate_declaration_ts_inner(
        summary,
        Some(ScriptContent::Split {
            first: plain_script,
            second: setup_script,
            second_start: plain_script.len() as u32 + 1,
        }),
    )
}

fn generate_declaration_ts_inner(
    summary: &Croquis,
    script_content: Option<ScriptContent<'_>>,
) -> DeclarationTsOutput {
    let mut ts = String::default();
    let generic_param = generic_param(summary);
    let generic_decl = generic_param
        .map(|generic| cstr!("<{}>", add_generic_defaults(generic)))
        .unwrap_or_default();
    let generic_ref = generic_param
        .map(|generic| cstr!("<{}>", extract_generic_names(generic)))
        .unwrap_or_default();

    if let Some(script) = script_content.as_ref() {
        emit_module_statements(&mut ts, summary, script);
    }

    emit_props_type(&mut ts, summary, generic_decl.as_str());
    emit_emits_type(&mut ts, summary, generic_decl.as_str());
    emit_slots_type(&mut ts, summary, generic_decl.as_str());
    emit_default_component(&mut ts, generic_decl.as_str(), generic_ref.as_str());

    DeclarationTsOutput { content: ts }
}

enum ScriptContent<'a> {
    Single(&'a str),
    Split {
        first: &'a str,
        second: &'a str,
        second_start: u32,
    },
}

impl<'a> ScriptContent<'a> {
    fn get(&self, start: u32, end: u32) -> Option<&'a str> {
        match self {
            Self::Single(script) => script.get(start as usize..end as usize),
            Self::Split {
                first,
                second,
                second_start,
            } => {
                if end <= *second_start {
                    return first.get(start as usize..end as usize);
                }
                if start >= *second_start {
                    let start = (start - *second_start) as usize;
                    let end = (end - *second_start) as usize;
                    return second.get(start..end);
                }
                None
            }
        }
    }
}

fn emit_module_statements(ts: &mut String, summary: &Croquis, script: &ScriptContent<'_>) {
    let mut spans: SmallVec<[(u32, u32); 8]> = SmallVec::new();
    for import in &summary.import_statements {
        spans.push((import.start, import.end));
    }
    for re_export in &summary.re_exports {
        spans.push((re_export.start, re_export.end));
    }
    for type_export in &summary.type_exports {
        spans.push((type_export.start, type_export.end));
    }

    spans.sort_unstable();
    spans.dedup();

    for (start, end) in spans {
        let Some(text) = script.get(start, end) else {
            continue;
        };
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        ts.push_str(text);
        ts.push('\n');
    }

    if !ts.is_empty() {
        ts.push('\n');
    }
}

fn emit_props_type(ts: &mut String, summary: &Croquis, generic_decl: &str) {
    if type_exists(summary, "Props") {
        return;
    }

    if let Some(type_args) = summary
        .macros
        .define_props()
        .and_then(|call| call.type_args.as_ref())
    {
        let inner = strip_outer_angle_brackets(type_args.as_str());
        append!(*ts, "export type Props{generic_decl} = {inner};\n");
        return;
    }

    if !summary.macros.props().is_empty() {
        append!(*ts, "export type Props{generic_decl} = {{\n");
        for prop in summary.macros.props() {
            let key = property_key(prop.name.as_str());
            let optional = if prop.required { "" } else { "?" };
            let prop_type = prop.prop_type.as_deref().unwrap_or("unknown");
            append!(*ts, "  {key}{optional}: {prop_type};\n");
        }
        ts.push_str("};\n");
        return;
    }

    append!(*ts, "export type Props{generic_decl} = {{}};\n");
}

fn emit_emits_type(ts: &mut String, summary: &Croquis, generic_decl: &str) {
    if type_exists(summary, "Emits") {
        return;
    }

    if let Some(type_args) = summary
        .macros
        .define_emits()
        .and_then(|call| call.type_args.as_ref())
    {
        let inner = strip_outer_angle_brackets(type_args.as_str());
        append!(*ts, "export type Emits{generic_decl} = {inner};\n");
        return;
    }

    if !summary.macros.emits().is_empty() {
        append!(*ts, "export type Emits{generic_decl} = {{\n");
        for emit in summary.macros.emits() {
            let key = string_literal(emit.name.as_str());
            let payload = emit.payload_type.as_deref().unwrap_or("any[]");
            append!(*ts, "  {key}: {payload};\n");
        }
        ts.push_str("};\n");
        return;
    }

    append!(*ts, "export type Emits{generic_decl} = {{}};\n");
}

fn emit_slots_type(ts: &mut String, summary: &Croquis, generic_decl: &str) {
    if type_exists(summary, "Slots") {
        return;
    }

    if let Some(type_args) = summary
        .macros
        .define_slots()
        .and_then(|call| call.type_args.as_ref())
    {
        let inner = strip_outer_angle_brackets(type_args.as_str());
        append!(*ts, "export type Slots{generic_decl} = {inner};\n");
        return;
    }

    append!(*ts, "export type Slots{generic_decl} = {{}};\n");
}

fn emit_default_component(ts: &mut String, generic_decl: &str, generic_ref: &str) {
    let props_ref = cstr!("Props{generic_ref}");
    let emits_ref = cstr!("Emits{generic_ref}");
    let slots_ref = cstr!("Slots{generic_ref}");

    ts.push_str("type __EmitShape<T> = T extends (...args: any[]) => any ? T : T extends Record<string, any> ? {\n");
    ts.push_str("  [K in keyof T]: T[K] extends (...args: infer A) => any ? A : T[K] extends any[] ? T[K] : any[];\n");
    ts.push_str("} : Record<string, any[]>;\n");
    ts.push_str("type __EmitArgs<T, K extends keyof T> = T[K] extends any[] ? T[K] : any[];\n");
    ts.push_str("type __EmitFn<T> = __EmitShape<T> extends (...args: any[]) => any ? __EmitShape<T> : (<K extends keyof __EmitShape<T>>(event: K, ...args: __EmitArgs<__EmitShape<T>, K>) => void);\n");
    append!(*ts, "type __VizeComponentInstance{generic_decl} = {{\n");
    append!(*ts, "  $props: {props_ref};\n");
    append!(*ts, "  $emit: __EmitFn<{emits_ref}>;\n");
    append!(*ts, "  $slots: {slots_ref};\n");
    ts.push_str("};\n");
    append!(
        *ts,
        "declare const __vize_component__: new {generic_decl}(...args: any[]) => __VizeComponentInstance{generic_ref};\n"
    );
    ts.push_str("export default __vize_component__;\n");
}

fn generic_param(summary: &Croquis) -> Option<&str> {
    summary
        .scopes
        .iter()
        .find_map(|scope| match (scope.kind, scope.data()) {
            (ScopeKind::ScriptSetup, ScopeData::ScriptSetup(data)) => {
                data.generic.as_ref().map(|generic| generic.as_str())
            }
            _ => None,
        })
}

fn type_exists(summary: &Croquis, name: &str) -> bool {
    summary
        .type_exports
        .iter()
        .any(|type_export| type_export.name.as_str() == name)
}

fn property_key(name: &str) -> String {
    if is_identifier_name(name) {
        return name.into();
    }
    string_literal(name)
}

fn is_identifier_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first == '$' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

fn string_literal(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('\'', "\\'");
    cstr!("'{escaped}'")
}

fn strip_outer_angle_brackets(value: &str) -> &str {
    let value = value.trim();
    if !value.starts_with('<') {
        return value;
    }

    let mut depth = 0i32;
    for (index, ch) in value.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 && index == value.len() - 1 {
                    return &value[1..index];
                }
            }
            _ => {}
        }
    }

    value
}

fn extract_generic_names(generic_param: &str) -> String {
    let mut names = String::default();
    let mut depth = 0i32;
    let mut current = String::default();

    for ch in generic_param.chars() {
        match ch {
            '<' => {
                depth += 1;
                current.push(ch);
            }
            '>' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                append_generic_name(&mut names, current.trim());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    append_generic_name(&mut names, current.trim());
    names
}

fn append_generic_name(names: &mut String, param: &str) {
    if param.is_empty() {
        return;
    }
    let name = param.split_whitespace().next().unwrap_or(param);
    if !names.is_empty() {
        names.push_str(", ");
    }
    names.push_str(name);
}

fn add_generic_defaults(generic_param: &str) -> String {
    let mut result = String::default();
    let mut depth = 0i32;
    let mut current = String::default();

    for ch in generic_param.chars() {
        match ch {
            '<' => {
                depth += 1;
                current.push(ch);
            }
            '>' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                append_param_with_default(&mut result, current.trim());
                result.push_str(", ");
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    append_param_with_default(&mut result, current.trim());
    result
}

fn append_param_with_default(result: &mut String, param: &str) {
    if param.is_empty() {
        return;
    }
    result.push_str(param);

    let mut depth = 0i32;
    let has_default = param.chars().any(|ch| {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            '=' if depth == 0 => return true,
            _ => {}
        }
        false
    });
    if !has_default {
        result.push_str(" = any");
    }
}

#[cfg(test)]
mod tests {
    use super::generate_declaration_ts;
    use crate::{Analyzer, AnalyzerOptions};

    #[test]
    fn generates_type_macro_declaration() {
        let script = r#"import type { User } from './types'

interface PublicProps {
  user: User
  active?: boolean
}

const props = defineProps<PublicProps>()
const emit = defineEmits<{
  (event: 'select', user: User): void
}>()
const slots = defineSlots<{
  default(props: { user: User }): any
}>()
"#;

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        let summary = analyzer.finish();
        let output = generate_declaration_ts(&summary, Some(script));

        assert!(output
            .content
            .contains("import type { User } from './types'"));
        assert!(output.content.contains("interface PublicProps"));
        assert!(output.content.contains("export type Props = PublicProps;"));
        assert!(output.content.contains("export type Emits = {"));
        assert!(output.content.contains("export type Slots = {"));
        assert!(output
            .content
            .contains("export default __vize_component__;"));
    }

    #[test]
    fn generates_runtime_macro_declaration() {
        let script = r#"const props = defineProps({
  title: String,
  count: { type: Number, required: true },
  'data-id': String,
})
const emit = defineEmits(['save'])
"#;

        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        let summary = analyzer.finish();
        let output = generate_declaration_ts(&summary, Some(script));

        assert!(output.content.contains("title?: string;"));
        assert!(output.content.contains("count: number;"));
        assert!(output.content.contains("'data-id'?: string;"));
        assert!(output.content.contains("'save': any[];"));
    }
}
