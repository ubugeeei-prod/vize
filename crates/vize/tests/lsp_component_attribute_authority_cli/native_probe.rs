//! Every fresh rename fixture must prove the actual native checker is active.

use serde_json::json;

use super::Fixture;

pub(super) fn prove(fixture: &mut Fixture) {
    let invalid = "<script setup lang=\"ts\">\nconst nativeGuard: number = 'wrong';\n</script>\n<template>{{ nativeGuard }}</template>\n";
    let repaired = "<script setup lang=\"ts\">\nconst nativeGuard: number = 1;\n</script>\n<template>{{ nativeGuard }}</template>\n";
    std::fs::write(fixture.project.path().join("src/NativeGuard.vue"), invalid).unwrap();
    fixture.open_with_diagnostics(
        "src/NativeGuard.vue",
        invalid,
        1,
        json!([{
            "range":{"start":{"line":1,"character":6},"end":{"line":1,"character":17}},
            "severity":1,"code":2322,"source":"vize/types",
            "message":"Type 'string' is not assignable to type 'number'."
        }]),
    );
    fixture.change("src/NativeGuard.vue", repaired, 2, json!([]));
}
