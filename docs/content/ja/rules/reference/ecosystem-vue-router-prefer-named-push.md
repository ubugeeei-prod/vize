---
title: "ecosystem/vue-router-prefer-named-push"
description: "Vue Router のプログラムによる移動に名前付きルートを使います。"
---

# `ecosystem/vue-router-prefer-named-push`

Vue Router のプログラムによる移動に名前付きルートを使います。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

router.push に移動先のパスを文字列で渡し、URL の具体的なパスに依存しています。

```vue annotate="remove:2"
<script setup lang="ts">
router.push("/settings");
</script>
```

## 良い例

settings のルート名を持つオブジェクトを router.push に渡します。

```vue annotate="add:2"
<script setup lang="ts">
router.push({ name: "settings" });
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/vue-router-prefer-named-push": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [エコシステムと Nuxt のルール](../ecosystem.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_router_prefer_named_push.rs#L19)
