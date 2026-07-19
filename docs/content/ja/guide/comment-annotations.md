---
title: コメントの注釈
---

<!-- Generated translation; source: guide/comment-annotations.md -->

# コメントの注釈

Vize は、リンティング、診断、コード生成の動作を制御するためのコメントベースのアノテーションを提供します。使用される場所に応じて 2 つの注釈システムがあります。

- **`<!-- @vize:xxx -->`**— `<template>` の HTML コメント (Patina リンター ディレクティブ)
- **`// @vize forget: reason`**— `<script>` の JS コメント (ファイル間分析の抑制)

すべての `@vize:` テンプレート ディレクティブは**ビルド出力から削除**され、実稼働コードには決して表示されません。

## テンプレート ディレクティブ (`@vize:`)

`<template>` 内で HTML コメントとして使用されます。これらは Patina (組み込みリンター) の動作を制御します。

### `@vize:expected`

次の行で診断が行われることを期待してください。診断が生成されない場合、これは何も行われません。 `@ts-expect-error` に似ています。

```vue
<template>
  <ul>
    <!-- @vize:expected -->
    <li v-for="item in items">{{ item }}</li>
  </ul>
</template>
```

### `@vize:ignore-start` / `@vize:ignore-end`

領域内のすべての診断を抑制します。

```vue
<template>
  <!-- @vize:ignore-start -->
  <ul>
    <li v-for="item in items">{{ item }}</li>
  </ul>
  <!-- @vize:ignore-end -->
</template>
```

### `@vize:level(warn|error|off)`

次の行で診断の重大度をオーバーライドします。

```vue
<template>
  <!-- @vize:level(warn) -->
  <img src="/photo.png" />

  <!-- @vize:level(off) -->
  <li v-for="item in items">{{ item }}</li>
</template>
```

| 値      | 効果                   |
| ------- | ---------------------- |
| `warn`  | 警告に格下げ           |
| `error` | エラーにアップグレード |
| `off`   | 完全に抑制             |

### `@vize:todo`

TODO 警告を発行します。

```vue
<template>
  <!-- @vize:todo add loading state -->
  <div>{{ data }}</div>
</template>
```

### `@vize:fixme`

FIXME エラーを発行します。

```vue
<template>
  <!-- @vize:fixme broken on mobile -->
  <div class="layout">...</div>
</template>
```

### `@vize:deprecated`

非推奨の警告を発します。

```vue
<template>
  <!-- @vize:deprecated use NewComponent instead -->
  <OldComponent />
</template>
```

### `@vize:docs`

ドキュメントのコメント。糸くずの影響はありません。

```vue
<template>
  <!-- @vize:docs Primary action button for form submission -->
  <button type="submit">Submit</button>
</template>
```

### `@vize:dev-only`

運用ビルドで削除され、開発中に保持されるノードをマークします。

```vue
<template>
  <!-- @vize:dev-only -->
  <div class="debug-panel">{{ internalState }}</div>
</template>
```

### まとめ

| ディレクティブ           | 効果                                       | 重大度 |
| ------------------------ | ------------------------------------------ | ------ |
| `@vize:expected`         | 次の行に診断が表示されることが予想されます | —      |
| `@vize:ignore-start/end` | リージョン内のすべての診断を抑制します     | —      |
| `@vize:level(...)`       | 次の行の重大度を上書きする                 | —      |
| `@vize:todo <msg>`       | TODOを送信する                             | 警告   |
| `@vize:fixme <msg>`      | FIXMEを発行する                            | エラー |
| `@vize:deprecated <msg>` | 非推奨の通知を発行する                     | 警告   |
| `@vize:docs <text>`      | ドキュメント (糸くずの影響なし)            | —      |
| `@vize:dev-only`         | 生産中のストリップ                         | —      |

## スクリプト抑制 (`@vize forget`)

`<script>` 内で JS コメントとして使用されます。次の行のファイル間分析の警告 (クロッキー) を抑制します。

### 構文

```vue
<script setup>
// @vize forget: <reason>
<suppressed line>
</script>
```

- \*理由が必要です\*\*— 抑制が必要な理由を説明する必要があります。

### 例

```vue
<script setup>
import { inject } from "vue";

// @vize forget: intentionally destructuring for one-time read
const { count } = inject("state");
</script>
```

アノテーションがないと、Vize はリアクティブな `inject()` 戻り値を構造化するとリアクティブの追跡が中断されると警告します。

### ルール

| ルール       | 説明                                                                      |
| ------------ | ------------------------------------------------------------------------- |
| 必要な理由   | 理由のない `// @vize forget` はエラーです。                               |
| コロンは必須 | `// @vize forget: <reason>` (理由の前にコロン) を使用する必要があります。 |
| 次の行のみ   | 次のコメントではない、空ではない行に適用されます。                        |
| 孤児はいない | エラー後のコードのないファイルの末尾の抑制。                              |

### 複数の抑制

各 `@vize forget` は次のコード行に独立して適用されます。

```vue
<script setup>
import { inject } from "vue";

// @vize forget: one-time read for display name
const { name } = inject("user");

// @vize forget: static config value
const { theme } = inject("config");
</script>
```

### コメントをスキップする

抑制の対象は次の**code**行で、コメントと空白行はスキップされます。

```vue
<script setup>
// @vize forget: read-only access
// This comment is skipped
const { count } = inject("state");
</script>
```

### 一般的な理由

| 理由                         | いつ使用するか                           |
| ---------------------------- | ---------------------------------------- |
| `intentionally non-reactive` | 値はリアクティブである必要はありません。 |
| `read-only access`           | 読み取りのみで、変更は追跡しません。     |
| `legacy code`                | 既知の問題。後でリファクタリングします。 |
| `third-party integration`    | 外部ライブラリで必要                     |

### 無効な例

```ts
// @vize forget
const { count } = inject("state");
// ^ Error: requires a reason

// @vize forget because I said so
const { count } = inject("state");
// ^ Error: requires a colon before the reason

// @vize forget:
const { count } = inject("state");
// ^ Error: reason cannot be empty
```
