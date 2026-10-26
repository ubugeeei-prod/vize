use tower_lsp::lsp_types::Url;
use vize_l0::cstr;

use super::setup_local_references;
use crate::ide::IdeContext;
use crate::server::ServerState;

#[test]
fn public_or_property_linked_bindings_keep_project_navigation() {
    for (filename, source, query) in [
        (
            "App.vue",
            "<script setup lang=\"ts\">import { item } from './shared';</script><template>{{ item }}</template>",
            "item }",
        ),
        (
            "App.vue",
            "<script setup lang=\"ts\">const { item } = defineProps<{ item: number }>();</script><template>{{ item }}</template>",
            "item }",
        ),
        (
            "App.vue",
            "<script setup lang=\"ts\">const item = 1; defineExpose({ item });</script><template>{{ item }}</template>",
            "item =",
        ),
        (
            "App.vue",
            "<script setup lang=\"ts\">const item = defineModel<number>();</script><template>{{ item }}</template>",
            "item =",
        ),
        (
            "App.vue",
            "<script setup lang=\"ts\">const { item } = useStore();</script><template>{{ item }}</template>",
            "item }",
        ),
        (
            "App.vue",
            "<script>export const item = 1;</script><script setup>const item = 2;</script><template>{{ item }}</template>",
            "item = 2",
        ),
        (
            "App.art.vue",
            "<script setup>const item = 1;</script><template>{{ item }}</template>",
            "item =",
        ),
        (
            "App.vue",
            "<script setup>const item = 1;</script><template>{{ item }}</template>\n<art title=\"A\" component=\"./Child.vue\">\n<variant name=\"First\" default><p>{{ item }}</p></variant>\n</art>\n",
            "item =",
        ),
        (
            "App.vue",
            "<script setup>const item = 1;</script><template><Child :item=\"item\" /></template>",
            "item=\"",
        ),
        (
            "App.vue",
            "<script setup lang=\"ts\">const item = ;</script><template>{{ item }}</template>",
            "item =",
        ),
    ] {
        let state = ServerState::new();
        let uri = Url::parse(&cstr!("file:///{filename}")).unwrap();
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);
        let ctx = IdeContext::new(&state, &uri, source.find(query).unwrap()).unwrap();
        assert_eq!(setup_local_references(&ctx, true), None, "{source}");
        assert!(!state.has_corsa_bridge());
    }
}
