/** Practical SFC examples: one source packet owns the displayed code and live demo. */
import { readFileSync } from "node:fs";
import path from "node:path";
import { previewImageUrl } from "../../../../docs/previews/ui/screenshot-file.ts";
import { COMPOSABLE_CATALOG } from "../../../compose/core/src/catalog.ts";
import { blocks, table } from "./markdown.ts";
import { asyncEffectExamples } from "./async-effect-examples.ts";
import { browserEffectExamples } from "./browser-effect-examples.ts";
import { formStateExamples } from "./form-state-examples.ts";
import { domInteractionExamples } from "./dom-interaction-examples.ts";
import { observedBrowserExamples } from "./observed-browser-examples.ts";
import { scopedInputExamples } from "./scoped-input-examples.ts";
import { preferenceBrowserExamples } from "./preference-browser-examples.ts";
import { environmentExamples } from "./environment-examples.ts";
import { pointerSelectionExamples } from "./pointer-selection-examples.ts";
import { localeFormatExamples } from "./locale-format-examples.ts";
import { workflowStateExamples } from "./workflow-state-examples.ts";
import { timeFormatExamples } from "./time-format-examples.ts";
import { dataStateExamples } from "./data-state-examples.ts";
import { browserPageExamples } from "./browser-page-examples.ts";
import { deviceExamples } from "./device-examples.ts";

const stateExamples = [
  {
    name: "async-resource",
    title: "Request a delivery quote with cancellation",
    purpose:
      "Let a shopper change delivery service or cancel a quote request while keeping the last successful price visible.",
    observe:
      "Start Standard, then immediately Express. The slow Standard loader is aborted and only Express appears. Cancel request stops a pending producer without adding a response. Unavailable area shows a failure while retaining the last quote; choose a service to recover. Reset quote aborts pending work and clears the result.",
    context:
      "This is delayed local sample data, not a server integration: Standard takes 800 ms and the other choices take 300 ms. The producer listens to the real `AbortSignal`, clears its timer on abort and removes its listener when it settles. In an application pass that signal to `fetch`. `keepData` and `cancelPrevious` default to true. `execute()` resolves to a success, error, cancelled or superseded result; a loader rejection does not reject that promise. The response/cancellation counters and last execution label are example-owned history, so reset does not clear them. Call inside setup: no loader runs during SSR, and disposing the component scope aborts active work through the same signal. The package remains experimental.",
  },
  {
    name: "virtual-list",
    title: "Browse a thousand guides",
    purpose:
      "Keep a long, searchable catalogue responsive by rendering only the rows near its visible area.",
    observe:
      "Focus the list and use Page Down, Home or End, or activate Last result. The rendered positions move through 1,000 guides while only a few rows exist at once. Search for 099 to see eleven matches, then try missing for the empty state. Searching returns to the first match without moving focus from the input.",
    context:
      "The complete example uses local sample data, not a server request. Each row must occupy exactly the declared 40 CSS pixels, including padding and borders. `overscan: 2` keeps two additional rows on either side; the container's 200px height determines the visible area. Bind `containerProps` to the scroll region and `wrapperProps.style` to its inner list, then render `list` rather than the entire source. `scrollTo()` takes a zero-based position in the filtered list. The first seven rows render on the server; measurement and scroll tracking belong to the component scope. Only currently rendered rows are available to browser Find and screen readers. Position and total-count attributes describe them, but do not make the other rows available: use search to narrow the catalogue, and prefer pagination or a complete list when every item must be readable or contain focusable controls.",
  },
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
      "Destructure the returned refs before using them in a template. Wire `onBlur` to the native blur event. The error records the last blur; subsequent edits wait for another blur, so an empty recorded error does not promise that an unvalidated edit is ready. The example checks name length, not server availability; no validation runs during setup.",
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

export const composableExamples = [
  ...stateExamples,
  ...asyncEffectExamples,
  ...browserEffectExamples,
  ...formStateExamples,
  ...domInteractionExamples,
  ...observedBrowserExamples,
  ...scopedInputExamples,
  ...preferenceBrowserExamples,
  ...environmentExamples,
  ...pointerSelectionExamples,
  {
    name: "use-throttled",
    title: "Preview a release title while editing",
    purpose:
      "Keep the latest release-title draft editable while limiting how often its local preview updates.",
    observe:
      "Change the title once to update the preview immediately. Further edits in the same 800 ms window wait; the newest draft appears when that window ends. Update preview now applies the waiting draft immediately. Cancel waiting update keeps both your draft and the last accepted preview; the next edit starts a fresh window.",
    context:
      "The example uses useThrottled's default leading and trailing edges and its real timer. `pending` describes a queued update, not whether the draft equals the preview. `flush()` applies waiting work and closes its window; `cancel()` discards waiting work and closes its window without changing the draft. SSR mirrors the source without starting timers. Call inside setup so scope disposal releases the watcher and timer. This local preview neither saves nor publishes a title; the package remains experimental.",
  },
  ...localeFormatExamples,
  ...workflowStateExamples,
  ...timeFormatExamples,
  ...dataStateExamples,
  ...browserPageExamples,
  ...deviceExamples,
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
    `<details><summary>Compare the initial and interacted states</summary><div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(min(260px,100%),1fr));gap:16px"><figure style="margin:0"><img src="${previewImageUrl(`composables/${name}-720-initial.png`)}" alt="${example.title}: initial state" loading="lazy" style="width:100%" /><figcaption>Initial state</figcaption></figure><figure style="margin:0"><img src="${previewImageUrl(`composables/${name}-720-interacted.png`)}" alt="${example.title}: state after the verified interactions" loading="lazy" style="width:100%" /><figcaption>After the verified interactions</figcaption></figure></div></details>`,
    name === "use-storage"
      ? `<details><summary>When browser storage is unavailable</summary><p>Choices still change in this tab. Reloading restores Getting started because the browser cannot save the choice. This capture uses actual Chromium with localStorage disabled.</p><img src="${previewImageUrl("composables/use-storage-unavailable-360-interacted.png")}" alt="Publishing selected in this tab while browser storage is unavailable" loading="lazy" width="360" style="max-width:100%" /></details>`
      : "",
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
      ? "入力、入力に追従するプレビュー、検証、検索、履歴、ページ切り替え、非同期処理、タイマー、保存、画面サイズ、フォーカス、フォーム、手順、選択、循環する案内、クリップボード、スクロール、要素のサイズと表示範囲、通信状態、配色、画面幅に応じた表示、ポインター座標、ドラッグ、文書のテキスト選択を実例で試せます。各リンクのプレビューと表示コードは同じ Vue SFC を使っています (英語)。"
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
      ? `以下は全公開エントリーのソース由来リファレンスです。上記の ${composableExamples.length} 例以外については、ライブプレビューでの検証をまだ行っていません。`
      : `The source-generated reference below covers every public entry. Live preview interaction checks currently cover the ${composableExamples.length} examples above; other entries retain their API examples and runtime contracts.`,
  );
}
