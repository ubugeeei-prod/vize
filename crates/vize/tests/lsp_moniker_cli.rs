#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
//! `textDocument/moniker` names the symbol go-to-definition resolves.
use crate::support::lsp_vue_project as support;
use serde_json::json;
use support::Fixture;

const APP: &str = "<script setup lang=\"ts\">
import { count } from \"./lib\";
import Card from \"./Card.vue\";
const local = count + 1;
</script>

<template>
  <Card>{{ local }} {{ count }}</Card>
</template>
";

#[test]
fn monikers_name_package_exports_components_and_document_locals() {
    let mut fixture = Fixture::new_with_vue(APP);
    fixture.write_file("package.json", r#"{"name":"moniker-app","private":true}"#);
    fixture.write_file("lib.ts", "export const count = 1;\n");
    fixture.write_file(
        "Card.vue",
        "<template><section><slot /></section></template>\n",
    );
    fixture.open(APP);

    assert_eq!(
        fixture.request("textDocument/moniker", APP, "count }}</Card>"),
        json!([{ "scheme": "tsc", "identifier": "moniker-app:lib:count", "unique": "group", "kind": "export" }])
    );
    assert_eq!(
        fixture.request("textDocument/moniker", APP, "Card>{{"),
        json!([{ "scheme": "tsc", "identifier": "moniker-app:Card.vue:default", "unique": "group", "kind": "export" }])
    );
    assert_eq!(
        fixture.request("textDocument/moniker", APP, "local }}"),
        json!([{ "scheme": "tsc", "identifier": "App.vue:local:3:6", "unique": "document", "kind": "local" }])
    );
    fixture.shutdown();
}
