---
title: グローバルプレビューコントロール
description: ブランド、コンポーネントのテーマ、言語などを Musea の全プレビューで共有します。
---

# グローバルプレビューコントロール

`toolbar` を設定すると、プロジェクト固有のコントロールを Musea のアドオンツールバーに追加できます。
変更はバリアント一覧、Props、アクセシビリティ、全画面、複数ビューポートの各 iframe に届きます。
Vue アプリはその場で更新されるため、グローバル値の変更で iframe 内の状態は失われません。

## コントロールを定義する

```ts
import { musea } from "@vizejs/vite-plugin-musea";

musea({
  previewSetup: "musea.preview.ts",
  toolbar: [
    {
      id: "brand",
      title: "ブランド",
      type: "select",
      options: ["default", "brand-a", "brand-b"],
      default: "default",
    },
    {
      id: "scheme",
      title: "コンポーネントのテーマ",
      type: "toggle",
      options: [
        { value: "light", label: "ライト", icon: "☀" },
        { value: "dark", label: "ダーク", icon: "☾" },
      ],
      default: "light",
    },
    {
      id: "locale",
      title: "言語",
      type: "select",
      options: [
        { value: "en", label: "English" },
        { value: "ja", label: "日本語" },
      ],
      default: "en",
    },
  ],
});
```

| フィールド | 動作                                                                                                              |
| ---------- | ----------------------------------------------------------------------------------------------------------------- |
| `id`       | 一意のキー。英字で始まり、以降は英字・数字・`_`・`-` を使えます。                                                 |
| `title`    | 表示ラベルとアクセシブルな名前。                                                                                  |
| `type`     | `select` は選択リスト、`toggle` はラベル付きの2つのボタン。                                                       |
| `options`  | 空でない文字列配列、または `{ value, label, icon? }` の配列。値は文字列・有限の数値・真偽値で、重複はできません。 |
| `default`  | 選択肢と型まで一致する初期値。                                                                                    |
| `icon`     | Unicode 記号などのテキスト。HTML や SVG の文字列もテキストとして表示します。                                      |

無効な設定はプラグインの読み込み時にエラーになります。Vue 3 と Vue 2.7 で利用できます。
`toolbar` を省略すると、従来のプレビュー動作を保ちます。
組み込みの Light/Dark ボタンは引き続きキャンバスの背景色を変更し、コンポーネントのテーマとは独立しています。

## プレビューに反映する

toolbar に1つ以上のコントロールがある場合、第2引数の context で同じ Vue ref を受け取れます。
初期値をマウント前に読み、変更は ref の監視で反映します。コントロールがなければ context は渡されません。
設定をまたいでセットアップを再利用する場合は、第2引数の有無を確認してください。
`app` だけを受け取る従来の関数もそのまま使えます。

```ts
// musea.preview.ts
import { watch, type App } from "vue";
import type { MuseaPreviewContext } from "@vizejs/vite-plugin-musea";
import { createI18n } from "vue-i18n";

export default function setup(app: App, context?: MuseaPreviewContext) {
  const i18n = createI18n({
    legacy: false,
    locale: context?.globals.value.locale === "ja" ? "ja" : "en",
    messages: { en: {}, ja: {} },
  });
  app.use(i18n);

  if (!context) return;
  const { globals } = context;

  const stop = watch(
    globals,
    (values) => {
      document.documentElement.dataset.brand = String(values.brand);
      document.documentElement.dataset.scheme = String(values.scheme);
      i18n.global.locale.value = values.locale === "ja" ? "ja" : "en";
    },
    { immediate: true },
  );
  app.onUnmount(stop);
}
```

監視関数の中で Vuetify のテーマ API、CSS カスタムプロパティ、密度、文字の方向などを更新できます。
Vue 2.7 では `app.onUnmount(stop)` の代わりに `app.$on("hook:destroyed", stop)` で監視を解除してください。
各 iframe が個別の ref を持ち、Musea が検証済みの同じ値を全 iframe に配信します。
Props 操作でアプリが再マウントされても、その iframe の現在の ref を引き継ぎます。

## 状態を共有・復元する

値はギャラリーの base path ごとにローカルストレージへ保存し、SPA のページ移動でも維持します。
URL の `museaGlobals` クエリには値の JSON を入れるため、現在のギャラリー URL を共有すると別のブラウザでも状態を再現できます。
URL に指定した値は保存値より優先します。未知のキーは無視し、無効な選択肢は設定した初期値に戻します。
他のクエリと URL のハッシュは保持します。

ローカルストレージが無効でも、変更の反映、ページ移動、共有 URL は動きます。
静的ビルドのギャラリーにも同じコントロールとプレビューセットアップを含めます。

動作例は `pnpm --dir examples/vite-musea gallery:globals` で試せます。ブランド・テーマ・言語を切り替えられます。
通常の example コマンドは既存の設定を使います。

グローバル値の全組み合わせを自動撮影する VRT は未実装です。既存の VRT は設定した初期値を使い、
コントロールを追加しただけではスクリーンショットのケースは増えません。
