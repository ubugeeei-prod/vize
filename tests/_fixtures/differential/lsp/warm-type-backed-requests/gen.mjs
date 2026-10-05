import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
const root = path.dirname(new URL(import.meta.url).pathname);
const N = Number(process.argv[2] ?? 300);
const src = path.join(root, "src");
rmSync(src, { recursive: true, force: true });
mkdirSync(path.join(src, "c"), { recursive: true });
mkdirSync(path.join(src, "lib"), { recursive: true });
for (let m = 0; m < Math.ceil(N / 3); m++) {
  writeFileSync(path.join(src, "lib", `mod${m}.ts`), `import { computed, ref, type Ref } from "vue";

export type Item${m} = { id: number; label: string; tags: readonly string[] };
export function useThing${m}(initial: number): { count: Ref<number>; double: Readonly<Ref<number>>; items: Ref<Item${m}[]>; add(label: string): void } {
  const count = ref(initial);
  const double = computed(() => count.value * 2);
  const items = ref<Item${m}[]>([]);
  function add(label: string): void {
    items.value.push({ id: items.value.length, label, tags: [] });
    count.value += 1;
  }
  return { count, double, items, add };
}
`);
}
for (let i = 0; i < N; i++) {
  const kids = [1, 2, 3].map(k => (i + k * 7) % N).filter(k => k !== i);
  const m = i % Math.ceil(N / 3);
  writeFileSync(path.join(src, "c", `Comp${i}.vue`), `<script setup lang="ts">
import { computed, ref } from "vue";
import { useThing${m} } from "../lib/mod${m}";
${kids.map(k => `import Comp${k} from "./Comp${k}.vue";`).join("\n")}

const { title, size = 1 } = defineProps<{ title: string; size?: number }>();
const emit = defineEmits<{ select: [id: number] }>();
const open = ref(false);
const thing = useThing${m}(size);
const label = computed(() => \`\${title} (\${thing.count.value})\`);
function toggle(next: boolean): void {
  open.value = next;
  emit("select", thing.items.value.length);
}
</script>

<template>
  <section :class="{ open }">
    <h2 @click="toggle(!open)">{{ label }}</h2>
    <ul v-if="open">
      <li v-for="item in thing.items.value" :key="item.id">{{ item.label }}</li>
    </ul>
${kids.map(k => `    <Comp${k} v-if="size < 0" :title="title" @select="toggle(false)" />`).join("\n")}
  </section>
</template>
`);
}
