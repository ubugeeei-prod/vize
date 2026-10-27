//! Complete segments come from independent lexical authored/generated anchors.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "independent complete map packets"
)]

use oxc_sourcemap::SourceMap;
use serde_json::{Value, json};
use vize_atelier_vapor::{
    VaporCompilerExperimentalOptions, VaporCompilerOptions, compile_vapor_with_experimental_options,
};
use vize_l0::Allocator;

fn coordinate(text: &str, offset: usize) -> (u32, u32) {
    let prefix = text.get(..offset).unwrap();
    (
        prefix.bytes().filter(|b| *b == b'\n').count() as u32,
        prefix.rsplit('\n').next().unwrap().encode_utf16().count() as u32,
    )
}

fn expected(code: &str, source: &str) -> Vec<(u32, u32, u32, u32, Option<String>)> {
    let after = |text: &str, prefix: &str| text.find(prefix).unwrap() + prefix.len();
    [
        (after(code, "<"), source.find("<button").unwrap() + 1, None),
        (
            code.find("type=").unwrap(),
            source.find("type=").unwrap(),
            None,
        ),
        (after(code, "type=\\\""), after(source, "type=\""), None),
        (
            code.find("style=").unwrap(),
            source.find(":style=").unwrap() + 1,
            None,
        ),
        (after(code, "style=\\\""), after(source, ":style=\""), None),
        (code.find("id=").unwrap(), source.find("id=").unwrap(), None),
        (after(code, "id=\\\""), after(source, "id=\""), None),
        (code.find("export function render").unwrap(), 0, None),
        (
            code.rfind("_ctx.label").unwrap(),
            source.rfind("label").unwrap(),
            Some("label".to_owned()),
        ),
    ]
    .into_iter()
    .map(|(generated, authored, name)| {
        let (dl, dc) = coordinate(code, generated);
        let (sl, sc) = coordinate(source, authored);
        (dl, dc, sl, sc, name)
    })
    .collect()
}

fn checked(
    code: &str,
    source: &str,
    raw: &str,
) -> Result<Vec<(u32, u32, u32, u32, Option<String>)>, String> {
    let document: Value = serde_json::from_str(raw).map_err(|e| e.to_string())?;
    if document["version"] != 3
        || document["sources"] != json!(["Original 日😀.vue"])
        || document["sourcesContent"] != json!([source])
    {
        return Err("source identity".into());
    }
    let map = SourceMap::from_json_string(raw).map_err(|e| e.to_string())?;
    let tokens = map
        .get_source_view_tokens()
        .map(|t| {
            (
                t.get_dst_line(),
                t.get_dst_col(),
                t.get_src_line(),
                t.get_src_col(),
                t.get_name().map(str::to_owned),
            )
        })
        .collect::<Vec<_>>();
    if tokens != expected(code, source) {
        return Err("complete lexical segment array".into());
    }
    Ok(tokens)
}

fn vlq(value: i64) -> String {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut value = (value.unsigned_abs() << 1) | u64::from(value < 0);
    let mut output = String::new();
    loop {
        let mut digit = value & 31;
        value >>= 5;
        if value != 0 {
            digit |= 32;
        }
        output.push(char::from(*alphabet.get(digit as usize).unwrap()));
        if value == 0 {
            return output;
        }
    }
}

fn mappings(tokens: &[(u32, u32, u32, u32, Option<String>)]) -> String {
    let (mut line, mut column, mut source_line, mut source_column) = (0, 0, 0, 0);
    let mut output = String::new();
    let mut first = true;
    for &(dl, dc, sl, sc, ref name) in tokens {
        while line < dl {
            output.push(';');
            line += 1;
            column = 0;
            first = true;
        }
        if !first {
            output.push(',');
        }
        first = false;
        for delta in [
            i64::from(dc) - column,
            0,
            i64::from(sl) - source_line,
            i64::from(sc) - source_column,
        ] {
            output.push_str(&vlq(delta));
        }
        if name.is_some() {
            output.push('A');
        }
        column = i64::from(dc);
        source_line = i64::from(sl);
        source_column = i64::from(sc);
    }
    output
}

#[test]
fn folded_css_has_real_complete_utf16_origins_and_rejects_mutations() {
    let mut packets = Vec::new();
    for (prefix, expression) in [
        ("", "{fontWeight:'700'}"),
        ("<!-- 日本語 😀 -->\r\n", "{fontWeight:'700'}"),
        ("<!-- 日本語 😀 -->", "{fontWeight:'700'}"),
        (
            "<!-- inserted -->\n<!-- 日本語 😀 -->\n",
            "{fontWeight:'700'}",
        ),
        // The parser decodes this original HTML operand, and both emitters
        // must escape its synthesized CSS again without losing its origin.
        (
            "<!-- 日本語 😀 -->",
            "{fontWeight:'700','--Brand':'a&amp;b&lt;&gt;'}",
        ),
    ] {
        for retained in [false, true] {
            let source = format!(
                "{prefix}<button type=\"button\" :style=\"{expression}\" id=\"own\">{{{{label}}}}</button>\r\n"
            );
            let allocator = Allocator::new();
            let output = compile_vapor_with_experimental_options(
                &allocator,
                &source,
                VaporCompilerOptions {
                    prefix_identifiers: true,
                    davinci_retained_lane: retained,
                    ..Default::default()
                },
                VaporCompilerExperimentalOptions {
                    source_map: true,
                    source_map_filename: Some("Original 日😀.vue".into()),
                    ..Default::default()
                },
            );
            assert!(output.error_messages.is_empty());
            if expression.contains("--Brand") {
                let static_source = source.replace(
                    &format!(":style=\"{expression}\""),
                    "style=\"font-weight:700;--Brand:a&amp;b&lt;&gt;;\"",
                );
                let reference = compile_vapor_with_experimental_options(
                    &allocator,
                    &static_source,
                    VaporCompilerOptions {
                        prefix_identifiers: true,
                        davinci_retained_lane: retained,
                        ..Default::default()
                    },
                    VaporCompilerExperimentalOptions::default(),
                );
                assert!(reference.error_messages.is_empty());
                assert_eq!(output.code, reference.code);
                assert_eq!(output.templates, reference.templates);
            }
            let raw = output.map.as_deref().unwrap();
            let tokens = checked(&output.code, &source, raw).unwrap();
            let document: Value = serde_json::from_str(raw).unwrap();
            assert_eq!(document["mappings"], mappings(&tokens));
            let mut negatives = Vec::new();
            for delta in [-1i64, 1] {
                let mut changed = tokens.clone();
                let style = changed.get_mut(4).unwrap();
                style.3 = (i64::from(style.3) + delta) as u32;
                let mut changed_map = document.clone();
                changed_map["mappings"] = json!(mappings(&changed));
                let changed_raw = serde_json::to_string(&changed_map).unwrap();
                assert!(checked(&output.code, &source, &changed_raw).is_err());
                negatives.push(changed_map);
            }
            // This direct template API owns a RootNode over the whole input,
            // beginning at zero even when a real element follows comments.
            // Zero is therefore valid for render(), but never for this CSS
            // aggregate whose actual expression begins inside the button.
            let mut wrong_root = tokens.clone();
            let style = wrong_root.get_mut(4).unwrap();
            style.2 = 0;
            style.3 = 0;
            let mut changed = document.clone();
            changed["mappings"] = json!(mappings(&wrong_root));
            assert!(
                checked(
                    &output.code,
                    &source,
                    &serde_json::to_string(&changed).unwrap()
                )
                .is_err()
            );
            negatives.push(changed);
            let mut foreign = document.clone();
            foreign["sourcesContent"] = json!([format!("{source}<!-- foreign -->")]);
            assert!(
                checked(
                    &output.code,
                    &source,
                    &serde_json::to_string(&foreign).unwrap()
                )
                .is_err()
            );
            negatives.push(foreign);
            assert!(
                checked(
                    &output.code.replacen("<button", " <button", 1),
                    &source,
                    raw
                )
                .is_err()
            );
            packets.push(json!({"source":source,"retained":retained,"code":output.code,"map":document,"tokens":tokens,"negativeMaps":negatives}));
        }
    }
    if let Some(path) = std::env::var_os("VIZE_LITERAL_STYLE_MAP_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&packets).unwrap()).unwrap();
    }
}
