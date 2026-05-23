<script setup lang="ts">
import { ref } from "vue";
import FixturePanel from "./FixturePanel.vue";

const currentPanel = FixturePanel;
const showPanel = ref(true);
const canLoadMore = ref(false);

function fetchMore() {
  canLoadMore.value = false;
}
</script>

<template>
  <KeepAlive>
    <Suspense>
      <component :is="currentPanel" v-show="showPanel" />
    </Suspense>
  </KeepAlive>
  <button
    v-tooltip="'Load older items'"
    v-appear="showPanel ? fetchMore : null"
    v-show="canLoadMore"
  >
    Load more
  </button>
</template>
