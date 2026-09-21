<script setup lang="ts">
import { ref } from "vue";

const rows = [
  {
    id: "alpha",
    label: "Alpha",
    value: "one",
    field: "data-rank",
    style: { color: "red" },
    attrs: { draggable: "true" },
  },
  {
    id: "beta",
    label: "Beta",
    value: "two",
    field: "data-rank",
    style: { color: "blue" },
    attrs: { draggable: "false" },
  },
];

const isReady = ref(true);
const panelClass = ref("ready-panel");
const panelStyle = ref({ borderColor: "tomato" });
const panelLabel = "Rows";
const sectionRef = ref<HTMLElement | null>(null);

function activate(row: (typeof rows)[number]) {
  return row.id;
}
</script>

<template>
  <section
    ref="sectionRef"
    class="surface"
    :class="[panelClass, { ready: isReady }]"
    style="background: url(a;b); color: black"
    :style="panelStyle"
    :aria-label="panelLabel"
    :value.prop="rows.length"
    v-track="isReady"
  >
    <article
      v-for="row in rows"
      :key="row.id"
      v-bind="row.attrs"
      :[row.field].camel.prop="row.value"
      :style="row.style"
      @keyup.enter="activate(row)"
    >
      <button .value="row.value" :aria-label.attr="row.label" @click="activate(row)">
        {{ row.label }}
      </button>
    </article>

    <template v-if="rows.length > 1">
      <span>{{ rows.length }}</span>
    </template>
    <template v-else>
      <span>Empty</span>
    </template>
  </section>
</template>
