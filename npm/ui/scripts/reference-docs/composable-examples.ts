/** Practical SFC examples: one source packet owns the displayed code and live demo. */
import { readFileSync } from "node:fs";
import path from "node:path";
import { COMPOSABLE_CATALOG } from "../../../compose/core/src/catalog.ts";
import { blocks, table } from "./markdown.ts";

export const composableExamples = [
  {
    name: "use-toggle",
    title: "Show delivery details",
    purpose:
      "Reveal additional information while keeping the disclosure button's accessible state synchronized.",
    observe:
      "Activate the button with Enter or Space. The delivery details appear, the label changes, and `aria-expanded` follows the same boolean ref.",
    context:
      "`state` is writable owned state; `toggle()` inverts it. Calling `toggle(false)` closes it explicitly. No browser globals or cleanup are needed.",
  },
  {
    name: "use-counter",
    title: "Reserve workshop seats",
    purpose:
      "Keep a seat quantity between one and five and explain why further changes are unavailable.",
    observe:
      "Add seats until the limit message appears. Add and remove controls disable at their respective bounds; Reset restores the initial one-seat reservation.",
    context:
      "Mutate the readonly `count` through its controls. `atMin` and `atMax` are computed refs. Bounds are fixed when the composable is created.",
  },
  {
    name: "use-debounced",
    title: "Search the guide catalogue",
    purpose:
      "Update local search results only after typing stops, with explicit pending, flush, cancel, and empty states.",
    observe:
      "Type component. The input changes immediately while the result list waits 500 ms. Search now applies it immediately; Cancel keeps the last settled results. Try a term with no matches.",
    context:
      "`query` is the writable source; `debounced` is its readonly delayed view. This example filters local data and sends no network request. Call inside setup so its watcher and timer belong to the component scope. SSR mirrors the source without starting a timer.",
  },
  {
    name: "use-field",
    title: "Validate a display name",
    purpose:
      "Validate a profile field on blur and expose visited, changed, error, and reset states.",
    observe:
      "Enter Al and press Tab to see the validation message. Replace it with Ada and leave the field again to clear the error. Reset clears both the value and interaction state.",
    context:
      "Destructure the returned refs before using them in a template. Wire `onBlur` to the native blur event. The example checks name length, not server availability; no validation runs during setup.",
  },
  {
    name: "use-history",
    title: "Edit and undo a release title",
    purpose:
      "Give a text editor bounded undo/redo and group a multi-write operation into one undo step.",
    observe:
      "Edit the title and try Undo and Redo. Apply publication title performs its two assignments as one step. A new edit after Undo discards redo history; Clear history retains the title.",
    context:
      "Each ref assignment is recorded synchronously. The example keeps at most ten undo entries. Object mutations require reassignment and an appropriate clone function. Its watcher and retained history follow the component scope.",
  },
  {
    name: "use-offset-pagination",
    title: "Browse a guide catalogue",
    purpose:
      "Slice a local list into pages with previous/next boundaries and a writable page-size selector.",
    observe:
      "Go to the last page, then change Guides per page to five. The page is clamped to the new last page and the visible list changes. Previous and Next disable at the boundaries.",
    context:
      "`currentPage` is one-based; `offset` is zero-based. `currentPageSize` can be used with `v-model.number`. The composable manages state; the example's computed ref slices the data.",
  },
] as const;

export function composableExampleSource(packageRoot: string, name: string): string {
  if (!COMPOSABLE_CATALOG.entries.some((entry) => entry.subpath === `./${name}`))
    throw new Error(`Example has no public composable entry: ${name}`);
  return readFileSync(path.join(packageRoot, "examples", `${name}.vue`), "utf8");
}

export function composablePreviewMarkup(name: string, title: string): string {
  const url = `/component-previews/app/index.html?composable=${encodeURIComponent(name)}`;
  return blocks(
    `<iframe src="${url}" title="${title} interactive example" loading="lazy" width="100%" height="540" style="border:1px solid #8885;border-radius:8px"></iframe>`,
    `[Open the interactive example](${url})`,
  );
}

export function composableExampleSection(packageRoot: string, name: string): string {
  const example = composableExamples.find((item) => item.name === name);
  if (example == null) return "";
  return blocks(
    "## Try it",
    `### ${example.title}`,
    example.purpose,
    composablePreviewMarkup(name, example.title),
    `**What to observe:** ${example.observe}`,
    example.context,
    `<details><summary>Compare the initial and interacted states</summary><div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(min(260px,100%),1fr));gap:16px"><figure style="margin:0"><img src="/component-previews/composables/${name}-720-initial.png" alt="${example.title}: initial state" loading="lazy" style="width:100%" /><figcaption>Initial state</figcaption></figure><figure style="margin:0"><img src="/component-previews/composables/${name}-720-interacted.png" alt="${example.title}: state after the verified interactions" loading="lazy" style="width:100%" /><figcaption>After the verified interactions</figcaption></figure></div></details>`,
    "## Copy the example",
    "Save this complete SFC in your Vue project. Its public imports and source are the same ones executed in the preview.",
    ["```vue", composableExampleSource(packageRoot, name).trim(), "```"].join("\n"),
  );
}

export function composableHub(locale: string, linkPrefix: string): string {
  const ja = locale === "ja";
  return blocks(
    ja ? "## 動作から選ぶ" : "## Try a behavior",
    ja
      ? "入力、検証、検索、履歴、ページ切り替えを実例で試せます。各リンクのプレビューと表示コードは同じ Vue SFC を使っています (英語)。"
      : "See how reactive state changes an interface. Each live preview and its copyable code come from the same complete Vue SFC.",
    table(
      [ja ? "やりたいこと" : "Goal", "Composable"],
      composableExamples.map((example) => [
        `[${example.title}](${linkPrefix}${example.name}.md)`,
        `\`${example.name}\``,
      ]),
    ),
    ja ? "## 最小構成" : "## Minimal setup",
    "```bash\nvp install @vizejs/composable\n```",
    ja
      ? "Vue 3.5 以降のプロジェクトで、以下の各ページにある SFC をコピーしてください。タイマーや監視を持つ関数は `<script setup>` 内で呼び出し、コンポーネントのスコープでクリーンアップします。"
      : "Use a Vue 3.5+ project, then copy the complete SFC from an example page. Call lifecycle-bound utilities inside `<script setup>` so their watchers, listeners, and timers are disposed with the component.",
    ja ? "## 全 API" : "## All APIs",
    ja
      ? "以下は全公開エントリーのソース由来リファレンスです。上記の 6 例以外については、ライブプレビューでの検証をまだ行っていません。"
      : "The source-generated reference below covers every public entry. Live preview interaction checks currently cover the six examples above; other entries retain their API examples and runtime contracts.",
  );
}
