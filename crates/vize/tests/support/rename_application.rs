#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::string_slice, reason = "validated UTF-16 edit offsets")]

use serde_json::{Map, Value, json};

pub fn apply(sources: &Value, uris: &Value, reply: &Value) -> Result<Value, String> {
    if reply.get("error").is_some() {
        return Err(format!("whole rename error: {reply}"));
    }
    let result = reply
        .get("result")
        .ok_or_else(|| format!("missing whole rename result: {reply}"))?;
    if result.is_null() {
        return Ok(sources.clone());
    }
    let mut entries = Map::new();
    let object = result
        .as_object()
        .ok_or_else(|| format!("invalid whole workspace edit: {result}"))?;
    if object.len() == 1 && object.contains_key("changes") {
        entries = object["changes"]
            .as_object()
            .ok_or_else(|| format!("invalid changes: {result}"))?
            .clone();
    } else if object.len() == 1 && object.contains_key("documentChanges") {
        for document in object["documentChanges"]
            .as_array()
            .ok_or_else(|| format!("invalid documentChanges: {result}"))?
        {
            let doc = document
                .as_object()
                .ok_or_else(|| format!("invalid document: {document}"))?;
            if doc.len() != 2 || !doc.contains_key("textDocument") || !doc.contains_key("edits") {
                return Err(format!("unsupported whole operation: {document}"));
            }
            let uri = document["textDocument"]["uri"]
                .as_str()
                .ok_or_else(|| format!("missing document URI: {document}"))?;
            if entries
                .insert(uri.to_owned(), document["edits"].clone())
                .is_some()
            {
                return Err(format!("duplicate document transaction: {result}"));
            }
        }
    } else {
        return Err(format!("unsupported whole workspace edit: {result}"));
    }
    if entries
        .keys()
        .any(|uri| !uris.as_object().unwrap().values().any(|owned| owned == uri))
    {
        return Err(format!("unowned edit target: {result}"));
    }
    let mut applied = Map::new();
    for (name, source) in sources.as_object().unwrap() {
        let text = source.as_str().unwrap();
        let edits = entries
            .get(uris[name].as_str().unwrap())
            .cloned()
            .unwrap_or_else(|| json!([]));
        let mut spans = Vec::new();
        for edit in edits
            .as_array()
            .ok_or_else(|| format!("invalid edit array: {edits}"))?
        {
            let object = edit
                .as_object()
                .ok_or_else(|| format!("invalid edit: {edit}"))?;
            if object.len() != 2 || !object.contains_key("range") || !object.contains_key("newText")
            {
                return Err(format!("unsupported edit metadata: {edit}"));
            }
            let start = offset(text, &edit["range"]["start"])?;
            let end = offset(text, &edit["range"]["end"])?;
            let replacement = edit["newText"]
                .as_str()
                .ok_or_else(|| format!("invalid edit text: {edit}"))?;
            if start > end {
                return Err(format!("reversed edit: {edit}"));
            }
            spans.push((start, end, replacement));
        }
        spans.sort_by_key(|&(start, end, _)| (start, end));
        if spans.windows(2).any(|pair| pair[0].1 > pair[1].0) {
            return Err(format!("overlapping actual edits: {edits}"));
        }
        let mut text = text.to_owned();
        for &(start, end, replacement) in spans.iter().rev() {
            text.replace_range(start..end, replacement);
        }
        applied.insert(name.to_owned(), json!(text));
    }
    Ok(json!(applied))
}

fn offset(text: &str, point: &Value) -> Result<usize, String> {
    let invalid = || format!("invalid authored UTF-16 position: {point}");
    let line = point["line"].as_u64().ok_or_else(invalid)? as usize;
    let character = point["character"].as_u64().ok_or_else(invalid)? as usize;
    let selected = text.split('\n').nth(line).ok_or_else(invalid)?;
    let selected = selected.strip_suffix('\r').unwrap_or(selected);
    let start: usize = text.split_inclusive('\n').take(line).map(str::len).sum();
    let mut units = 0;
    for (index, ch) in selected.char_indices() {
        if units == character {
            return Ok(start + index);
        }
        units += ch.len_utf16();
    }
    if units == character {
        Ok(start + selected.len())
    } else {
        Err(invalid())
    }
}
