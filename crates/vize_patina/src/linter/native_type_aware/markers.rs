use super::parsing::extract_runtime_object_property_values;
use vize_carton::{String, ToCompactString};
use vize_croquis::virtual_ts::VirtualTsOutput;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum QueryKind {
    PropType,
    EmitValidator,
    Promise,
}

pub(super) struct MacroQuery {
    pub kind: QueryKind,
    pub generated_offset: u32,
    pub source_start: u32,
    pub source_end: u32,
}

pub(super) fn push_promise_marker(
    virtual_ts: &mut VirtualTsOutput,
    script_content: &str,
    source_start: u32,
    source_end: u32,
    queries: &mut Vec<MacroQuery>,
) {
    let Some(insert_offset) = marker_insert_offset(&virtual_ts.content) else {
        return;
    };
    let source_range = source_start as usize..source_end as usize;
    let Some(expression_source) = script_content.get(source_range) else {
        return;
    };

    let mut marker_name = String::with_capacity(28);
    marker_name.push_str("__vize_patina_promise_");
    let query_index = queries.len().to_compact_string();
    marker_name.push_str(query_index.as_str());

    let mut line = String::with_capacity(marker_name.len() + expression_source.len() + 40);
    line.push_str("    function ");
    let name_offset = line.len() as u32;
    line.push_str(&marker_name);
    line.push_str("() { return (");
    line.push_str(expression_source);
    line.push_str("); }\n");

    let generated_offset = insert_offset as u32 + name_offset;
    virtual_ts.content.insert_str(insert_offset, &line);
    queries.push(MacroQuery {
        kind: QueryKind::Promise,
        generated_offset,
        source_start,
        source_end,
    });
}

pub(super) fn push_prop_type_markers(
    virtual_ts: &mut VirtualTsOutput,
    runtime_args: Option<&str>,
    source_start: u32,
    source_end: u32,
    queries: &mut Vec<MacroQuery>,
) {
    let Some(runtime_args) = runtime_args else {
        return;
    };
    let Some(insert_offset) = marker_insert_offset(&virtual_ts.content) else {
        return;
    };
    let prop_sources = extract_runtime_object_property_values(runtime_args);
    if prop_sources.is_empty() {
        return;
    }

    let mut block = String::with_capacity(runtime_args.len() + 128);
    for (index, prop_source) in prop_sources.iter().enumerate() {
        let mut prop_name = String::with_capacity(24);
        prop_name.push_str("__vize_patina_prop_");
        prop_name.push_str(queries.len().to_compact_string().as_str());
        prop_name.push('_');
        prop_name.push_str(index.to_compact_string().as_str());

        block.push_str("    const ");
        let name_offset = block.len() as u32;
        block.push_str(&prop_name);
        block.push_str(" = (undefined as unknown as __RuntimePropCtor<typeof (");
        block.push_str(prop_source.as_str());
        block.push_str(")>);\n");

        queries.push(MacroQuery {
            kind: QueryKind::PropType,
            generated_offset: insert_offset as u32 + name_offset,
            source_start,
            source_end,
        });
    }

    virtual_ts.content.insert_str(insert_offset, &block);
}

pub(super) fn push_emit_validator_markers(
    virtual_ts: &mut VirtualTsOutput,
    runtime_args: Option<&str>,
    source_start: u32,
    source_end: u32,
    queries: &mut Vec<MacroQuery>,
) {
    let Some(runtime_args) = runtime_args else {
        return;
    };
    let Some(insert_offset) = marker_insert_offset(&virtual_ts.content) else {
        return;
    };

    let validator_sources = extract_runtime_object_property_values(runtime_args);
    if validator_sources.is_empty() {
        return;
    }

    let mut block = String::with_capacity(runtime_args.len() + 128);
    for (index, validator_source) in validator_sources.iter().enumerate() {
        let mut validator_name = String::with_capacity(24);
        validator_name.push_str("__vize_patina_emit_");
        validator_name.push_str(queries.len().to_compact_string().as_str());
        validator_name.push('_');
        validator_name.push_str(index.to_compact_string().as_str());

        block.push_str("    const ");
        let name_offset = block.len() as u32;
        block.push_str(&validator_name);
        block.push_str(" = (");
        block.push_str(validator_source.as_str());
        block.push_str(");\n");

        queries.push(MacroQuery {
            kind: QueryKind::EmitValidator,
            generated_offset: insert_offset as u32 + name_offset,
            source_start,
            source_end,
        });
    }

    virtual_ts.content.insert_str(insert_offset, &block);
}

fn marker_insert_offset(content: &str) -> Option<usize> {
    content
        .rfind("\n}\n\n// Invoke setup")
        .map(|index| index + 1)
}
