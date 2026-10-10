use vize_carton::FxHashMap;

use super::support::{Fixture, immutable_documents, same_mtime_write};

const BEFORE: &str = "// é🦀\r\nexport const importedProps = { oldName: { type: String, required: true } } as const;\r\n";
const AFTER: &str = "// é🦀\r\nexport const importedProps = { newName: { type: String, required: true } } as const;\r\n";
const GRAPH: &str = "export const importedProps = { graphOnly: Boolean } as const;\n";

fn assert_revision(fixture: &Fixture, host: &str, relative: &str) {
    let empty = FxHashMap::default();
    let first = fixture.open(host, &empty);
    let mirror = first.mirror.as_ref().unwrap();
    let target = fixture.root.join(relative);
    assert!(mirror.find_by_original(&target).is_some());
    let before = mirror.find_by_original(&fixture.host).unwrap();
    assert!(
        before
            .content
            .contains("  const oldName = props[\"oldName\"]")
    );
    assert!(
        !before
            .content
            .contains("  const newName = props[\"newName\"]")
    );
    let old_catalog = first.source_catalog.clone();
    let old_documents = first.materialized_sources();
    let whole_old = immutable_documents(&first);
    drop(first);
    same_mtime_write(&target, AFTER);
    assert_eq!(fixture.guard(host, &empty).unwrap(), vec![target]);
    let updated = fixture.open(host, &empty);
    assert!(
        updated.graph_reused,
        "all retained mapped Vue originals are regenerated"
    );
    let mirror = updated.mirror.as_ref().unwrap();
    let after = mirror.find_by_original(&fixture.host).unwrap();
    assert!(
        after
            .content
            .contains("  const newName = props[\"newName\"]")
    );
    assert!(
        !after
            .content
            .contains("  const oldName = props[\"oldName\"]")
    );
    assert!(
        !after
            .content
            .contains("  const graphOnly = props[\"graphOnly\"]")
    );
    let mapped = mirror
        .registered_original_paths_sorted()
        .iter()
        .filter(|path| {
            mirror
                .find_by_original(path)
                .is_some_and(|file| file.source_map.sfc_map.is_some())
        })
        .count();
    assert_eq!(mirror.source_patch_work(), (mapped + 1, mapped + 1));
    fixture.assert_cold_equal(&updated, host, &empty);
    // The old catalog stays immutable even though its physical namespace is reused.
    let retained = old_documents
        .into_iter()
        .map(|source| {
            let old = old_catalog.get(&source.materialized_path).unwrap();
            assert_eq!(old.source, source.source);
            assert_eq!(old.code, source.code);
            assert_eq!(old.mapping, source.mapping);
            assert_eq!(old.import_source_map, source.import_source_map);
            source
        })
        .collect::<Vec<_>>();
    assert_eq!(retained.len(), whole_old.len());
}

#[test]
fn unchanged_relative_runtime_props_importer_regenerates_its_new_key_surface() {
    let host = "<script setup lang='ts'>import { importedProps } from './props'; const props = defineProps(importedProps);</script><template>{{ props }}</template>\n";
    let fixture = Fixture::new(host);
    fixture.write("src/props.ts", BEFORE);
    assert_revision(&fixture, host, "src/props.ts");
}

#[test]
fn alias_codegen_input_outside_its_recorded_import_edge_regenerates_whole_surface() {
    let host = "<script setup lang='ts'>import Child from './Child.vue'; import { importedProps } from '@/props'; const props = defineProps(importedProps);</script><template>{{ props }}<Child/></template>\n";
    let fixture = Fixture::new(host);
    fixture.write("tsconfig.json", "{\"compilerOptions\":{\"strict\":true,\"baseUrl\":\".\",\"paths\":{\"@/*\":[\"graph/*\"]}},\"include\":[\"src/**/*\",\"graph/**/*\"]}\n");
    fixture.write("graph/props.ts", GRAPH);
    fixture.write("src/props.ts", BEFORE);
    fixture.write("src/Child.vue", "<script setup lang='ts'>import { importedProps } from './props'; void importedProps;</script><template>child</template>\n");
    // The graph resolves @/props to graph/props.ts. Existing Vue runtime-prop
    // augmentation independently consumes src/props.ts, reached through Child.
    assert_revision(&fixture, host, "src/props.ts");
}

#[test]
fn workspace_codegen_input_outside_package_route_edge_regenerates_whole_surface() {
    let host = "<script setup lang='ts'>import Child from './Child.vue'; import { importedProps } from '@scope/props'; const props = defineProps(importedProps);</script><template>{{ props }}<Child/></template>\n";
    let fixture = Fixture::new(host);
    fixture.write(
        "package.json",
        "{\"name\":\"runtime-prop-control\",\"private\":true}\n",
    );
    fixture.write(
        "node_modules/@scope/props/package.json",
        "{\"name\":\"@scope/props\",\"types\":\"index.ts\"}\n",
    );
    fixture.write("node_modules/@scope/props/index.ts", GRAPH);
    fixture.write("packages/props/index.ts", BEFORE);
    fixture.write("src/Child.vue", "<script setup lang='ts'>import { importedProps } from '../packages/props'; void importedProps;</script><template>child</template>\n");
    // Package route discovery resolves the installed declaration; runtime-prop
    // augmentation independently consumes the authored ancestor packages tree.
    assert_revision(&fixture, host, "packages/props/index.ts");
}
