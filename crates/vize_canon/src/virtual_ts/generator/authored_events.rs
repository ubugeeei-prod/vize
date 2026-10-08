use vize_carton::{FxHashSet, String, append, cstr};
use vize_croquis::Croquis;

use crate::virtual_ts::macro_type_mappings::MacroTypeMappings;

pub(super) fn emit_authored_event_map(
    ts: &mut String,
    summary: &Croquis,
    mappings: &mut MacroTypeMappings<'_>,
    generic_decl: Option<&str>,
    generic_names: &str,
) {
    let events = summary.macros.emits();
    let models = summary.macros.models();
    let authored_map = generic_decl.map_or_else(
        || String::from("__VizeAuthoredEventMap"),
        |decl| cstr!("__VizeAuthoredEventMap<{decl}>"),
    );
    let generic_suffix = generic_decl
        .map(|_| cstr!("<{generic_names}>"))
        .unwrap_or_default();
    // Preserve the original property symbols (and their JSDoc) alongside the
    // mapped navigation endpoints. Reconstructing a fresh object loses docs.
    append!(
        *ts,
        "type {authored_map} = __VizeStaticEventMap{generic_suffix} & {{\n"
    );
    let mut emitted_names = FxHashSet::default();
    for emit in events {
        if !emitted_names.insert(emit.name.as_str()) {
            continue;
        }
        let Some(authored_range) = summary.macros.emit_declaration(emit.name.as_str()) else {
            continue;
        };
        let Some(authored_name) = mappings.authored_text(authored_range).map(String::from) else {
            continue;
        };
        ts.push_str("  ");
        let generated_start = ts.len();
        ts.push_str(authored_name.as_str());
        let generated_end = ts.len();
        append!(*ts, ": __VizeStaticEventMap{generic_suffix}[");
        ts.push_str(
            serde_json::to_string(emit.name.as_str())
                .unwrap_or_default()
                .as_str(),
        );
        ts.push_str("];\n");
        let quoted = authored_name
            .as_bytes()
            .first()
            .copied()
            .is_some_and(|quote| {
                matches!(quote, b'\'' | b'"')
                    && authored_name.as_bytes().last() == Some(&quote)
                    && authored_name.get(1..authored_name.len().saturating_sub(1))
                        == Some(emit.name.as_str())
                    && !emit.name.is_empty()
            });
        if quoted {
            // Croquis retains the exact AST literal, and generated text keeps
            // its quotes. Native definitions cover that whole literal; native
            // references/edits use its interior. Retain both producer endpoints.
            let content = (authored_range.0 + 1, authored_range.1 - 1);
            mappings.map_whole_symbol(generated_start..generated_end, content);
            mappings.map_exact(generated_start + 1..generated_end - 1, content);
        } else {
            mappings.map_exact(generated_start..generated_end, authored_range);
        }
    }
    let mut emitted_models = FxHashSet::default();
    for model in models {
        if !emitted_models.insert(model.name.as_str()) {
            continue;
        }
        let event_name = cstr!("update:{}", model.name);
        if emitted_names.contains(event_name.as_str()) {
            continue;
        }
        let Some(authored_range) = summary.macros.model_declaration(model.name.as_str()) else {
            continue;
        };
        ts.push_str("  /* __vize_model_event */ ");
        let generated_start = ts.len();
        ts.push_str(
            serde_json::to_string(event_name.as_str())
                .unwrap_or_default()
                .as_str(),
        );
        let generated_end = ts.len();
        append!(*ts, ": __VizeStaticEventMap{generic_suffix}[");
        ts.push_str(
            serde_json::to_string(event_name.as_str())
                .unwrap_or_default()
                .as_str(),
        );
        ts.push_str("];\n");
        mappings.map_whole_symbol(generated_start..generated_end, authored_range);
    }
    ts.push_str("};\n");
}
