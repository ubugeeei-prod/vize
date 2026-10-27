export type InputCase = {
  id: string;
  source: string;
  missing?: boolean;
  beforeEmits?: number;
  emits?: number;
  render?: boolean;
  html?: string;
};
const script = `const emit = defineEmits<{ press: [] }>();`;
const component = (setup: string, template: string): string =>
  `<script setup lang="ts">\n${script}\n${setup}\n</script>\n<template>${template}</template>`;
export const templateGlobalCases: InputCase[] = [
  {
    id: "focus",
    source: component(
      "",
      `<button @click="(event: MouseEvent) => { if (document.activeElement !== event.currentTarget) return; emit('press'); }">press</button>`,
    ),
    missing: true,
    beforeEmits: 1,
  },
  {
    id: "explicit",
    source: component(
      "",
      `<button @click="(event: MouseEvent) => { if (globalThis.document.activeElement !== event.currentTarget) return; emit('press'); }">press</button>`,
    ),
    missing: true,
    beforeEmits: 1,
  },
  {
    id: "allowed",
    source: component(
      "",
      `<button @click="(event: MouseEvent) => { if (Math.max(1, 2) !== 2 || Intl.NumberFormat('en-US').format(2) !== '2') return; emit('press'); }">press</button>`,
    ),
    emits: 1,
  },
  {
    id: "callback",
    source: component(
      "",
      `<button @click="(document: MouseEvent) => { if (!document.currentTarget) return; emit('press'); }">press</button>`,
    ),
    emits: 1,
  },
  {
    id: "setup",
    source: component(
      "const document = { activeElement: null };",
      `<button @click="(event: MouseEvent) => { if (document.activeElement !== event.currentTarget) return; emit('press'); }">press</button>`,
    ),
    emits: 0,
  },
  {
    id: "loop",
    source: component(
      "",
      `<div><button v-for="document in [{ activeElement: null }]" @click="(event: MouseEvent) => { if (document.activeElement !== event.currentTarget) return; emit('press'); }">press</button></div>`,
    ),
    emits: 0,
  },
  {
    id: "render-unknown",
    source: component("", `<p>{{ document.activeElement }}{{ Math.max(1, 2) }}</p>`),
    missing: true,
    render: true,
  },
  {
    id: "render-setup",
    source: component(
      "const document = { activeElement: 'setup' };",
      `<p>{{ document.activeElement }}{{ Math.max(1, 2) }}</p>`,
    ),
    emits: 0,
    render: true,
    html: "<p>setup2</p>",
  },
];
