use vize_carton::FxHashMap;

use super::support::Fixture;

const WITHOUT: &str =
    "<script setup lang='ts'>const value = 1;</script><template>{{ value }}</template>\n";
const WITH: &str = "<script setup lang='ts'>import Target from './Target.vue';</script><template><Target/></template>\n";

#[test]
fn released_then_reacquired_known_overlay_cannot_be_replaced_by_disk_registration() {
    let host = "<script setup lang='ts'>import A from './A.vue'; import B from './B.vue';</script><template><A/><B/></template>\n";
    let fixture = Fixture::new(host);
    let a = fixture.write("src/A.vue", WITHOUT);
    let b = fixture.write("src/B.vue", WITH);
    let target = fixture.write(
        "src/Target.vue",
        "<script setup lang='ts'>const value = 'disk';</script><template>{{ value }}</template>\n",
    );
    let unsaved = "<!-- 🦀 -->\r\n<script setup lang='ts'>const value = 'unsaved';</script>\r\n<template>{{ value }}</template>\r\n";
    let overlays = FxHashMap::from_iter([(target.clone(), unsaved)]);
    let first = fixture.open(host, &overlays);
    let mirror = first.mirror.as_ref().unwrap();
    let file = mirror.find_by_original(&target).unwrap();
    assert_eq!(
        mirror.original_content_for_virtual(&file.virtual_path),
        Some(unsaved)
    );
    drop(first);
    std::fs::write(&a, WITH).unwrap();
    std::fs::write(&b, WITHOUT).unwrap();
    assert!(fixture.guard(host, &overlays).is_ok());
    let transferred = fixture.open(host, &overlays);
    assert!(
        !transferred.graph_reused,
        "post-walk supplied authority must catch reacquisition from disk"
    );
    let mirror = transferred.mirror.as_ref().unwrap();
    let file = mirror.find_by_original(&target).unwrap();
    assert_eq!(
        mirror.original_content_for_virtual(&file.virtual_path),
        Some(unsaved)
    );
    assert!(file.content.contains("'unsaved'"));
    assert!(!file.content.contains("'disk'"));
    fixture.assert_cold_equal(&transferred, host, &overlays);
}
