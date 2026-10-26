//! Independent decoded-map and mutation oracles for backend-origin tests.
use oxc_sourcemap::SourceMap;
use serde_json::{Value, json};

fn byte_at(text: &str, line: u32, column: u32) -> Option<usize> {
    if line > text.bytes().filter(|&byte| byte == b'\n').count() as u32 {
        return None;
    }
    let start = text
        .split_inclusive('\n')
        .take(line as usize)
        .map(str::len)
        .sum::<usize>();
    let mut units = 0;
    for (offset, ch) in text.get(start..)?.char_indices() {
        if units == column {
            return Some(start + offset);
        }
        if ch == '\n' {
            return None;
        }
        units += ch.len_utf16() as u32;
        if units > column {
            return None;
        }
    }
    (units == column).then_some(text.len())
}

/// Every segment, independently located in complete code and authored input.
pub(super) fn check_map(
    code: &str,
    source: &str,
    raw: &str,
    backend: &str,
) -> Result<Value, &'static str> {
    let document: Value = serde_json::from_str(raw).map_err(|_| "invalid map JSON")?;
    if document["version"] != 3
        || document["sources"] != json!(["Original 日本語.tsx"])
        || document["sourcesContent"] != json!([source])
    {
        return Err("map source identity changed");
    }
    let map = SourceMap::from_json_string(raw).map_err(|_| "invalid source map")?;
    let tokens = map
        .get_source_view_tokens()
        .map(|token| {
            let generated = byte_at(code, token.get_dst_line(), token.get_dst_col())
                .ok_or("invalid generated coordinate")?;
            let authored = byte_at(source, token.get_src_line(), token.get_src_col())
                .ok_or("invalid authored coordinate")?;
            Ok(json!({ "generated": generated, "authored": authored, "name": token.get_name() }))
        })
        .collect::<Result<Vec<_>, &'static str>>()?;
    let root = source.find("<p>").ok_or("fixture root missing")?;
    let label = source.rfind("label").ok_or("fixture expression missing")?;
    let literal = code.find("<p").ok_or("generated literal missing")? + 1;
    let expression = code.rfind("label").ok_or("generated expression missing")?;
    let expected = match backend {
        "vapor" => json!([
            { "generated": literal, "authored": root + 1, "name": null },
            { "generated": code.find("export function render").ok_or("render missing")?, "authored": root, "name": null },
            { "generated": expression, "authored": label, "name": null },
        ]),
        "ssr" => json!([
            { "generated": code.find("function ssrRender").ok_or("ssrRender missing")?, "authored": root, "name": null },
            { "generated": literal, "authored": root + 1, "name": null },
            { "generated": expression, "authored": label, "name": null },
        ]),
        _ => return Err("unknown backend"),
    };
    if json!(tokens) != expected {
        return Err("complete decoded segments differ from independent lexical origins");
    }
    Ok(json!({ "code": code, "map": document, "tokens": tokens }))
}

fn vlq(value: i64) -> String {
    let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut value = (value.unsigned_abs() << 1) | u64::from(value < 0);
    let mut output = String::new();
    loop {
        let mut digit = value & 31;
        value >>= 5;
        if value != 0 {
            digit |= 32;
        }
        output.push(char::from(*table.get(digit as usize).unwrap()));
        if value == 0 {
            return output;
        }
    }
}

pub(super) fn rejects_mutations(code: &str, source: &str, raw: &str) -> Vec<Value> {
    let document: Value = serde_json::from_str(raw).unwrap();
    let root = source.find("<p>").unwrap();
    let prefix = source.get(..root).unwrap();
    let line = prefix.bytes().filter(|&byte| byte == b'\n').count() as i64;
    let column = prefix.rsplit('\n').next().unwrap().encode_utf16().count() as i64;
    let (_, rest) = document["mappings"]
        .as_str()
        .unwrap()
        .split_once(';')
        .unwrap();
    let mut negatives = Vec::new();
    for (name, line, column) in [
        ("synthetic-module-start", 0, 0),
        ("root-minus-one", line, column - 1),
        ("root-plus-one", line, column + 1),
    ] {
        let mut changed = document.clone();
        changed["mappings"] = json!(format!("AA{}{};{rest}", vlq(line), vlq(column)));
        let raw = serde_json::to_string(&changed).unwrap();
        assert!(check_map(code, source, &raw, "ssr").is_err(), "{name}");
        negatives.push(json!({ "id": name, "code": code, "map": changed }));
    }
    let mut foreign = document.clone();
    foreign["sourcesContent"] = json!([format!("{source}// foreign root")]);
    assert!(
        check_map(
            code,
            source,
            &serde_json::to_string(&foreign).unwrap(),
            "ssr"
        )
        .is_err()
    );
    negatives.push(json!({ "id": "foreign-source", "code": code, "map": foreign }));
    let moved = format!("\n{code}");
    assert!(check_map(&moved, source, raw, "ssr").is_err());
    negatives.push(json!({ "id": "moved-generated-code", "code": moved, "map": document }));
    negatives
}
