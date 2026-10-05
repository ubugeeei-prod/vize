## Version

- `vize` 0.432.0 (npm, `node_modules/.bin/vize lsp --stdio`; bundled Corsa `@typescript/typescript-darwin-arm64` 7.0.2), serverInfo `vize-maestro 0.432.0`
- `vue` 3.5.41, Node 26.6.0, macOS (arm64, 15 cores, 48 GB)
- For comparison: `tsserver` 5.9.3 + Vue - Official 3.3.6's TypeScript plugin (`vue-typescript-plugin-pack`, the hybrid-mode setup the VS Code extension uses)

The machine is shared and was busy (load average 75–130 during the runs, printed with each run). Absolute times are inflated by that; the comparison was run back-to-back under the same load.

## Reproduction

A generator for ordinary SFCs (each imports 3 others and one composable module):

`gen.mjs`:

```js
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
```

`tsconfig.json`: `target`/`module` `ESNext`, `moduleResolution: "Bundler"`, `strict`, `jsx: "preserve"`, `skipLibCheck`, `include: ["src/**/*.ts", "src/**/*.vue"]`. `vize.config.json`: `{ "languageServer": { "typecheck": true, "editor": true } }`. `package.json` depends on `vue` 3.5.41.

Steps: `node gen.mjs <N>`, start `vize lsp --stdio`, `initialize`, `didOpen src/c/Comp0.vue`, wait for the type diagnostics (`finished initial type diagnostics` in the log), then 5 rounds of: `hover` on `open` (`const open = ref(false)`), `hover` on `label` in `{{ label }}`, `definition` on `useThing0`, `completion` after `thing.` in `thing.count.value`. Times are request → response.

## Actual

`vize lsp` (warm, after the first type diagnostics):

```
20 SFCs   didOpen -> type diagnostics 12.5 s   footprint  307 MiB
  hover          367 / 356 / 131 / 621 / 477 ms
  definition     465 / 802 /  54 /  44 / 925 ms
  completion     490 /  52 / 181 / 301 /  62 ms
100 SFCs  didOpen -> type diagnostics 40.3 s   footprint  837 MiB
  hover          1079 / 1663 / 1460 / 1411 / 1285 ms
  definition     1983 / 1704 / 2069 / 1063 / 2360 ms
  completion     1332 / 1895 / 1486 / 1763 / 1760 ms
  hoverTemplate  1614 /  936 / 1302 / 1913 / 1026 ms
400 SFCs  didOpen -> type diagnostics 186.7 s  footprint 2814 MiB
  hover          7219 / 4342 / 4614 / 3976 / 5498 ms
  definition     4499 / 3251 / 2858 / 4297 / 6690 ms
  completion     2226 / 3331 / 3835 / 5782 / 5267 ms
  hoverTemplate  3918 / 5067 / 4352 / 2545 / 5029 ms
```

`tsserver` + Vue - Official's plugin, same files, run right after:

```
100 SFCs  open -> semantic diagnostics 23.5 s   footprint 378 MiB
  hover          696 / 2 / 4 / 2 / 1 ms
  definition      16 / 2 / 5 / 1 / 4 ms
  completion     490 / 4 / 6 / 1 / 6 ms
  hoverTemplate  177 / 4 / 6 / 3 / 2 ms
400 SFCs  open -> semantic diagnostics 41.3 s   footprint 542 MiB
  hover          167 / 467 / 9 / 1 / 2 ms
  definition     219 /  33 / 6 / 2 / 1 ms
  completion     405 /  23 / 6 / 2 / 4 ms
  hoverTemplate   31 /   7 / 5 / 3 / 2 ms
```

The repeated request on the same position does not get faster in Vize, and its cost grows roughly with the number of SFCs, as if each request rebuilt or re-sent the project to Corsa. Requests that do not need the checker stay fast (in the real app below: hover on a component tag 6 ms, `documentSymbol` 3 ms, `foldingRange` 4 ms, `semanticTokens` 3 ms, formatting 32 ms). The server log shows `WARN corsa_client::api::snapshot: failed to release corsa snapshot \`N\`: release queue is closed` around these requests.

In a real app of this size (~290 SFCs, ~850 `.ts`, Nuxt) the numbers match: warm hover / definition / signature help / completion 2–5 s each, the first type diagnostics for a 1,080-line page 80–100 s, an edit → new type diagnostics 36–115 s, and a 1,700-line page ran into the 60 s `lspRequestTimeoutMs` bound (`typecheck-timed-out`). The same page through tsserver + Vue - Official: 3 ms warm hover, 2 ms definition, 9 ms completion, 15 s after an edit.

## Expected

After the first check, hover / definition / completion / signature help answer from the warm program in tens of milliseconds, independent of how many SFCs the project has, and an edit to one SFC re-checks only what depends on it. Memory in the same range as tsserver for the same project.

## Why

These are the requests an editor sends on every cursor move and keystroke; at 1–7 s each, `editor: true` cannot replace Vue - Official in a mid-size app even though the answers are now correct (#7192 / #7193 / #7194 are fixed — thanks). Related roadmap items: #6872 (keep LSP state light), #6850 (serve LSP from incremental artifacts), #7698 (type checker throughput). This issue is the measurable symptom for the warm path, with a generator to track it.
