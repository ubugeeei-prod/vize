/** Task-oriented entry to the source-generated UI reference. */
import { featuredExamples } from "./examples.ts";
import { uiFamilyCatalog } from "../../src/catalog/family-catalog.ts";
import { blocks } from "./markdown.ts";

export function uiHub(locale: string, linkPrefix: string): string {
  const ja = locale === "ja";
  const title = (family: string) =>
    uiFamilyCatalog.find((entry) => entry.canonicalName === family)!.title;
  const goals: [string, string[]][] = [
    [
      ja ? "入力を受け取る" : "Collect user input",
      ["input", "textarea", "checkbox", "switch", "select", "combobox"],
    ],
    [ja ? "操作を実行する" : "Run an action", ["button", "copy-button", "share-button", "toggle"]],
    [
      ja ? "情報を整理する" : "Organize a page",
      ["card", "tabs", "accordion", "separator", "breadcrumb"],
    ],
    [
      ja ? "確認・補足を表示する" : "Ask for confirmation or add context",
      ["dialog", "alert-dialog", "popover", "tooltip", "dropdown-menu"],
    ],
    [
      ja ? "結果・進捗を伝える" : "Show feedback and progress",
      ["alert", "toast", "badge", "progress", "skeleton"],
    ],
    [
      ja ? "データ・日付を扱う" : "Browse data and dates",
      ["table", "pagination", "calendar", "date-picker", "tree"],
    ],
  ];
  const example = [
    "```vue",
    '<script setup lang="ts">',
    'import { ref } from "vue";',
    'import { Button } from "@vizejs/ui/button";',
    "const saved = ref(false);",
    "</script>",
    "",
    "<template>",
    '  <Button type="button" @press="saved = true">Save changes</Button>',
    '  <output>{{ saved ? "Saved" : "Ready" }}</output>',
    "</template>",
    "```",
  ].join("\n");
  const cards = featuredExamples
    .map(
      (family) =>
        `<a href="${linkPrefix}${family}.md" style="display:block;color:inherit;text-decoration:none"><img src="/component-previews/${family}.png" alt="${title(family)} browser-rendered example" loading="lazy" style="width:100%;border:1px solid #8885;border-radius:8px" /><strong>${title(family)}</strong></a>`,
    )
    .join("\n");
  return blocks(
    ja ? "## 作りたい画面から選ぶ" : "## Choose what you want to build",
    ja
      ? "各リンクには、公開 import でコピーできる Vue の使用例、操作できるプレビュー、props・イベント・スロットの仕様があります。"
      : "Open a component for a copyable Vue example with public imports, an interactive preview, and its props, events, and slots.",
    `<div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(min(240px,100%),1fr));gap:16px">\n${goals.map(([goal, families]) => `<section style="padding:16px;border:1px solid #8885;border-radius:8px"><strong>${goal}</strong><p>${families.map((family) => `<a href="${linkPrefix}${family}.md">${title(family)}</a>`).join(" · ")}</p></section>`).join("\n")}\n</div>`,
    ja ? "## 最初のコンポーネント" : "## Add your first component",
    "```bash\nvp install @vizejs/ui\n```",
    ja
      ? "[Vite+ 連携 (英語)](/guide/vite-plus/)で Vue のコンパイルを設定したプロジェクトに、次の SFC を追加します。"
      : "Add this SFC to a project with Vue compilation configured through the [Vite+ integration](../vite-plus.md).",
    example,
    ja
      ? "コンポーネントの振る舞いはスタイルなしでも動作します。以下のプレビューは任意の Paper スタイルを読み込んでいます。[スタイルの導入とテーマ切り替え](../ui-styles.md)を参照してください。"
      : "The component behavior works without styles. These previews add the optional Paper styles; see [styles and theme switching](../ui-styles.md) to use the same appearance.",
    ja ? "## 実際に描画された使用例" : "## See the components in use",
    ja
      ? "画像は実際の Vue コンポーネントをブラウザーで描画して取得しています。画像から使用例を開き、入力・切り替え・ダイアログを試せます。"
      : "These screenshots come from the real Vue examples rendered in a browser. Open one to try typing, toggling state, or opening a dialog.",
    `<div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(230px,1fr));gap:20px">\n${cards}\n</div>`,
    ja ? "## 自分のコンポーネントも並べる" : "## Explore your own components",
    ja
      ? "[Musea](../musea.md) は `*.art.vue` の使用例をコンポーネントギャラリーにします。[コンポーザブル](../composables/index.md)で振る舞いを組み合わせ、[Source Distribution](../lib-pull.md)で必要なソースを取得できます。"
      : "[Musea](../musea.md) turns your `*.art.vue` examples into a component gallery. Use [composables](../composables/index.md) to compose behavior, or [Source Distribution](../lib-pull.md) to own a component's source.",
    ja ? "## 全コンポーネントの API" : "## All component APIs",
    ja
      ? "用途別にまとめた全ファミリーの仕様です。maturity は各ファミリーの現在のサポート段階を示します。"
      : "The complete source-generated reference follows, grouped by use case. Maturity describes each family's current support tier.",
  );
}
