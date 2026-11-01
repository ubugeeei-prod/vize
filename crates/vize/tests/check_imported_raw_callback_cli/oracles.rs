//! Exact source and cross-channel oracles for one derived Linux fixture.
use super::{CONFIG, I18N, PARENT};
use serde_json::{Value, json};
use std::{fs, path::Path};
use vize_l0::{String as CompactString, cstr};

pub(super) fn parent(kind: &str) -> CompactString {
    let direct = "  <p>{{ i18n.ts._role._priority.low }}</p>";
    let callback = "  <Formatter :textConverter=\"(v) => i18n.ts._role._priority.low\" />";
    assert_eq!(PARENT.matches(direct).count(), 1);
    assert_eq!(PARENT.matches(callback).count(), 1);
    match kind {
        "wrong" => PARENT.replace(direct,"  <p>{{ i18n.ts._role._priority.low.repeat('direct_bad') }}</p>")
            .replace(callback,"  <Formatter :textConverter=\"(v) => i18n.ts._role._priority.low.repeat('callback_bad')\" />").into(),
        "missing" => PARENT.replace(direct,"  <p>{{ i18n.ts._role._priority.directMissing }}</p>")
            .replace(callback,"  <Formatter :textConverter=\"(v) => i18n.ts._role._priority.callbackMissing\" />").into(),
        "" => PARENT.into(),
        other => panic!("unknown fixture state {other}"),
    }
}

fn markers(kind: &str) -> [(&'static str, u32, CompactString); 2] {
    match kind {
        "wrong" => [
            (
                "'direct_bad'",
                2345,
                "Argument of type 'string' is not assignable to parameter of type 'number'.".into(),
            ),
            (
                "'callback_bad'",
                2345,
                "Argument of type 'string' is not assignable to parameter of type 'number'.".into(),
            ),
        ],
        "missing" => [
            (
                "directMissing",
                2339,
                "Property 'directMissing' does not exist on type 'Priority'.".into(),
            ),
            (
                "callbackMissing",
                2339,
                "Property 'callbackMissing' does not exist on type 'Priority'.".into(),
            ),
        ],
        other => panic!("no negatives for {other}"),
    }
}

pub(super) fn expected_public(kind: &str, source: &str) -> (Value, CompactString) {
    if kind.is_empty() {
        return (json!([]), CompactString::default());
    }
    let mut mapped = Vec::new();
    let mut javascript = CompactString::default();
    for (index, (marker, code, message)) in markers(kind).into_iter().enumerate() {
        let positions = source
            .lines()
            .enumerate()
            .filter_map(|(line, text)| text.find(marker).map(|column| (line, text, column)))
            .collect::<Vec<_>>();
        assert_eq!(positions.len(), 1);
        let (line, text, column) = positions[0];
        let column = text[..column].encode_utf16().count() + 1;
        let context = if index == 0 {
            text.trim().into()
        } else {
            cstr!("{}; binding: 'textConverter'", text.trim())
        };
        mapped.push(cstr!(
            "error:{}:{} [TS{code}] {message} (source: {context})",
            line + 1,
            column
        ));
        javascript.push_str(
            cstr!(
                "src/Parent.vue({},{}): error TS{code}: {message}\n",
                line + 1,
                column
            )
            .as_str(),
        );
    }
    (json!(mapped), javascript)
}

pub(super) fn main_call(calls: &Value) -> &Value {
    let rows = calls
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["request"]["kind"] == "main-cli")
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 1, "{calls}");
    rows[0]
}

fn included<'a>(call: &'a Value, literal: &str) -> &'a Value {
    let entries = call["includedSourceBodies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["literal"] == literal)
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 1, "{call}");
    entries[0]
}

pub(super) fn body<'a>(calls: &'a Value, literal: &str) -> &'a str {
    included(main_call(calls), literal)["body"]
        .as_str()
        .unwrap()
}

pub(super) fn bytes(value: &Value) -> Vec<u8> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|n| u8::try_from(n.as_u64().unwrap()).unwrap())
        .collect()
}

pub(super) fn assert_native(calls: &Value, geometry: &Value, kind: &str) {
    let call = main_call(calls);
    let request = &call["request"];
    let cwd = Path::new(request["cwd"].as_str().unwrap());
    let actual_owner = Path::new(
        included(call, "src/Parent.vue.ts")["physical"]
            .as_str()
            .unwrap(),
    );
    assert_eq!(
        actual_owner,
        fs::canonicalize(cwd.join("src/Parent.vue.ts")).unwrap()
    );
    assert_eq!(
        geometry["imports"],
        json!([{"line":11,"column":23,"text":"./Formatter.vue.ts"}])
    );
    assert_eq!(geometry["parseDiagnostics"], json!([]));
    // CURRENT image4f2de/7.0.2 fixture debt, proven by f45's full body/config/raw; never native-clean.
    let mut expected:CompactString="src/Parent.vue.ts(11,23): error TS5097: An import path can only end with a '.ts' extension when 'allowImportingTsExtensions' is enabled.\n".into();
    let negatives = geometry["negative"].as_array().unwrap();
    let owners = negatives
        .iter()
        .map(|node| node["owner"].clone())
        .collect::<Vec<_>>();
    assert_eq!(
        owners,
        if kind.is_empty() {
            json!([])
        } else {
            json!(["interpolation", "inference-only", "checked-callback"])
        }
        .as_array()
        .unwrap()
        .to_vec()
    );
    if kind.is_empty() {
        assert_eq!(negatives, &Vec::<Value>::new());
    } else {
        for (index, (marker, code, message)) in markers(kind).into_iter().enumerate() {
            let selected = negatives
                .iter()
                .filter(|node| node["marker"] == marker)
                .collect::<Vec<_>>();
            assert_eq!(
                selected.len(),
                if index == 0 { 1 } else { 2 },
                "native emitted source changed owned marker multiplicity {marker}"
            );
            for node in selected {
                assert_eq!(node["code"], code);
                assert_eq!(node["message"], message.as_str());
            }
        }
    }
    for node in negatives {
        // The emitter's exact guarded inference copy is retained in geometry.
        // Only interpolation and the mapped callback owner report diagnostics.
        match node["owner"].as_str().unwrap() {
            "inference-only" => continue,
            "interpolation" | "checked-callback" => {}
            other => panic!("unknown emitted diagnostic owner {other}"),
        }
        expected.push_str(
            cstr!(
                "src/Parent.vue.ts({},{}): error TS{}: {}\n",
                node["line"],
                node["column"],
                node["code"],
                node["message"].as_str().unwrap()
            )
            .as_str(),
        );
    }
    assert_eq!(call["status"]["status"], 1, "{calls}");
    assert_eq!(call["status"]["signal"], Value::Null, "{calls}");
    assert_eq!(call["status"]["error"], Value::Null, "{calls}");
    assert_eq!(bytes(&call["stdout"]), expected.as_bytes(), "{calls}");
    assert_eq!(bytes(&call["stderr"]), b"", "{calls}");
}

pub(super) fn assert_public(
    packet: &Value,
    calls: &Value,
    kind: &str,
    source: &str,
    baseline: &Value,
) {
    let (diagnostics, _) = expected_public(kind, source);
    assert_eq!(body(calls, "src/i18n.ts"), I18N);
    assert_eq!(
        body(calls, "src/Formatter.vue.ts"),
        baseline["files"][0]["virtualTs"].as_str().unwrap()
    );
    let expected = json!({"files":[
        {"file":"src/Formatter.vue","virtualTs":body(calls,"src/Formatter.vue.ts"),"diagnostics":[]},
        {"file":"src/Parent.vue","virtualTs":body(calls,"src/Parent.vue.ts"),"diagnostics":diagnostics},
        {"file":"src/i18n.ts","virtualTs":I18N,"diagnostics":[]}
    ],"programs":[{"root":".","tsconfig":"tsconfig.json","compilerOptions":serde_json::from_str::<Value>(CONFIG).unwrap()["compilerOptions"],
        "files":["src/Formatter.vue","src/Parent.vue","src/i18n.ts"]}],"errorCount":if kind.is_empty(){0}else{2},"warningCount":0,"fileCount":3});
    assert_eq!(packet, &expected);
}
