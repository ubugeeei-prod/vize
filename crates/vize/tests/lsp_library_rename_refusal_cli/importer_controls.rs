//! Whole non-null importer edits preserve TS/JS and authored declarations.

use serde_json::json;

use super::{Fixture, byte_offset, range};

const CONFIG: &str = r#"{
  "compilerOptions": {
    "lib": ["ESNext", "DOM"], "strict": true, "moduleResolution": "Bundler",
    "module": "ESNext", "target": "ESNext", "allowJs": true, "checkJs": true
  },
  "include": ["src/**/*.vue", "src/**/*.ts", "src/**/*.js"]
}"#;
const PROBE: &str = "<script setup lang=\"ts\">\nimport { tsValue } from './types';\nimport { jsValue } from './values.js';\nimport { declaredValue } from './declared';\n</script>\n<template>{{ tsValue }} {{ jsValue }} {{ declaredValue }}</template>\n";
const TS: &str = "export const tsValue = 1;\n";
const JS: &str = "export const jsValue = 2;\n";
const DECLARED: &str = "export declare const declaredValue: number;\n";

#[test]
fn non_null_ts_js_and_authored_declaration_renames_keep_complete_importer_edits() {
    for crlf in [false, true] {
        let convert = |text: &str| {
            if crlf {
                text.replace('\n', "\r\n")
            } else {
                text.to_owned()
            }
        };
        let probe = convert(PROBE);
        let ts = convert(TS);
        let js = convert(JS);
        let declared = convert(DECLARED);
        let mut fixture = Fixture::with_config(
            &[
                ("src/Probe.vue", &probe),
                ("src/types.ts", &ts),
                ("src/values.js", &js),
                ("src/declared.d.ts", &declared),
            ],
            CONFIG,
        );
        fixture.open("src/Probe.vue", &probe, 1);
        let shifted = format!("<!-- 😀 -->{}{}", if crlf { "\r\n" } else { "\n" }, probe);
        for (version, source) in [
            (1, probe.as_str()),
            (2, shifted.as_str()),
            (3, probe.as_str()),
        ] {
            if version > 1 {
                fixture.change("src/Probe.vue", source, version, json!([]));
            }
            for (name, file, target) in [
                ("tsValue", "src/types.ts", ts.as_str()),
                ("jsValue", "src/values.js", js.as_str()),
                ("declaredValue", "src/declared.d.ts", declared.as_str()),
            ] {
                let renamed = format!("next{name}");
                let template_selector = format!("{{{{ {name} }}}}");
                let template_offset = source.find(&template_selector).unwrap() + 3;
                let import_selector = format!("import {{ {name} }}");
                let import_offset = source.find(&import_selector).unwrap() + "import { ".len();
                let target_selector = format!("const {name}");
                let target_offset = target.find(&target_selector).unwrap() + "const ".len();
                let expected = json!({"changes":{
                    fixture.uri(file):[{
                        "range":range(target,target_offset,name),"newText":renamed
                    }],
                    fixture.uri("src/Probe.vue"):[
                        {"range":range(source,import_offset,name),"newText":renamed},
                        {"range":range(source,template_offset,name),"newText":renamed}
                    ]
                }});
                for query in [import_offset, template_offset] {
                    let actual = fixture.request(
                        "src/Probe.vue",
                        "textDocument/rename",
                        range(source, query, name)["start"].clone(),
                        json!({"newName":renamed}),
                    );
                    let typed: lsp_types::WorkspaceEdit =
                        serde_json::from_value(actual.clone()).unwrap();
                    assert_eq!(actual, expected, "complete source/importer edit set");
                    let changes = typed.changes.unwrap();
                    assert_eq!(changes.len(), 2);
                    for (path, text) in [(file, target), ("src/Probe.vue", source)] {
                        let uri: lsp_types::Uri = fixture.uri(path).parse().unwrap();
                        let mut edits = changes[&uri].clone();
                        edits.sort_by_key(|edit| std::cmp::Reverse(edit.range.start));
                        let mut applied = text.to_owned();
                        for edit in edits {
                            applied.replace_range(
                                byte_offset(text, edit.range.start)
                                    ..byte_offset(text, edit.range.end),
                                &edit.new_text,
                            );
                        }
                        assert_eq!(applied, text.replace(name, &renamed));
                    }
                }
            }
        }
        fixture.shutdown();
    }
}
