use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn literal_template_slot_name_uses_index_signature_payload() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "literal-template-slot-name",
        &[
            (
                "src/CmsListTable.vue",
                r#"<script setup lang="ts" generic="T extends { name: string }">
type Header = { key: string };
defineProps<{ items: T[] }>();
defineSlots<{
  actions?(): unknown;
  [key: `item.${string}`]: ((props: { item: T; header: Header }) => unknown) | undefined;
}>();
</script>

<template>
  <slot
    v-for="item in items"
    :key="item.name"
    :name="`item.${item.name}`"
    :item="item"
    :header="{ key: item.name }"
  />
</template>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import CmsListTable from './CmsListTable.vue';

type User = { name: string; email: string };
const users: User[] = [];

function takesUser(value: User) {
  return value.email;
}
function takesHeader(value: { key: string }) {
  return value.key;
}
</script>

<template>
  <CmsListTable :items="users">
    <template #[`item.name`]="{ item, header }">
      {{ takesUser(item) }}{{ takesHeader(header) }}{{ item.name }}
    </template>
  </CmsListTable>
</template>
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };
    let _ = std::fs::remove_dir_all(&project_root);

    assert!(
        snapshot.is_empty(),
        "literal template slot names should resolve typed payloads: {snapshot:#?}"
    );
}
