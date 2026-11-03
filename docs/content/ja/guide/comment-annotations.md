---
title: コメントによる設定
---

<!-- Reviewed translation; source: guide/comment-annotations.md -->

<span id="コメントの注釈"></span>

# コメントによる設定

特定の行や範囲だけで lint の扱いを変えたい場合にコメントを使います。コメントがなければ、プロジェクトで設定したルールと重大度がそのまま適用されます。

- `<template>` 内の `<!-- @vize:xxx -->` は、lint 診断の扱いやビルド時の出力を指定します。
- `<script>` 内の `// @vize forget: reason` は、次のコード行に対するファイル間解析の警告を抑制します。

すべてのファイルで方針を変える場合は[lint の実行設定](./lint-execution.md)、診断の読み方と対応は[診断ガイド](./analysis-diagnostics.md)を参照してください。

テンプレート内のすべての `@vize:` ディレクティブは**ビルド出力から削除**され、本番コードには含まれません。

## テンプレートのディレクティブ (`@vize:`)

`<template>` 内の HTML コメントに記述し、組み込みリンターの動作を指定します。

### `@vize:expected`

次の行に診断が出ることを宣言します。`@ts-expect-error` に似ていますが、診断が出なくても追加のエラーは発生しません。

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

次の行に出る診断の重大度を上書きします。

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
| `warn`  | 警告として扱う           |
| `error` | エラーとして扱う |
| `off`   | 完全に抑制             |

### `@vize:todo`

TODO の警告を報告します。

```vue
<template>
  <!-- @vize:todo add loading state -->
  <div>{{ data }}</div>
</template>
```

### `@vize:fixme`

FIXME のエラーを報告します。

```vue
<template>
  <!-- @vize:fixme broken on mobile -->
  <div class="layout">...</div>
</template>
```

### `@vize:deprecated`

非推奨であることを警告します。

```vue
<template>
  <!-- @vize:deprecated use NewComponent instead -->
  <OldComponent />
</template>
```

### `@vize:docs`

説明用のコメントです。lint の動作には影響しません。

```vue
<template>
  <!-- @vize:docs Primary action button for form submission -->
  <button type="submit">Submit</button>
</template>
```

### `@vize:dev-only`

開発中だけ表示するノードを指定します。本番ビルドでは削除されます。

```vue
<template>
  <!-- @vize:dev-only -->
  <div class="debug-panel">{{ internalState }}</div>
</template>
```

### まとめ

| ディレクティブ           | 効果                                       | 重大度 |
| ------------------------ | ------------------------------------------ | ------ |
| `@vize:expected`         | 次の行に診断が出ることを宣言する | —      |
| `@vize:ignore-start/end` | 指定した範囲の診断を抑制する     | —      |
| `@vize:level(...)`       | 次の行の重大度を上書きする                 | —      |
| `@vize:todo <msg>`       | TODO を警告する                             | 警告   |
| `@vize:fixme <msg>`      | FIXME のエラーを報告する                            | エラー |
| `@vize:deprecated <msg>` | 非推奨の通知を発行する                     | 警告   |
| `@vize:docs <text>`      | 説明用コメント（lint に影響しない）            | —      |
| `@vize:dev-only`         | 本番ビルドから削除する                         | —      |

## スクリプト抑制 (`@vize forget`)

`<script>` 内の JavaScript コメントで、次のコード行に対するファイル間解析の警告を抑制します。

### 構文

```vue
<script setup>
// @vize forget: <reason>
<suppressed line>
</script>
```

- **理由を記載してください** — 抑制が必要な理由を説明します。

### 例

```vue
<script setup>
import { inject } from "vue";

// @vize forget: intentionally destructuring for one-time read
const { count } = inject("state");
</script>
```

このコメントがない場合、Vize は `inject()` のリアクティブな戻り値を分割代入すると、リアクティビティを追跡できなくなる可能性を警告します。

### ルール

| ルール       | 説明                                                                      |
| ------------ | ------------------------------------------------------------------------- |
| 理由が必要   | 理由のない `// @vize forget` はエラーになります。 |
| コロンは必須 | `// @vize forget: <reason>` (理由の前にコロン) を使用する必要があります。 |
| 次のコード行のみ | コメントや空行を読み飛ばし、次のコード行に適用します。 |
| 後続のコードが必要 | 抑制コメントの後にコードがなければエラーになります。                              |

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

抑制の対象は次の**コード行**です。コメントと空行は読み飛ばします。

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
| `third-party integration`    | 外部ライブラリとの連携のために必要                     |

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
