---
title: "vize:croquis/cf/circular-reactive-dependency"
---

# `vize:croquis/cf/circular-reactive-dependency`

reactive な計算が互いに循環して依存しています。

既定の重大度: context-dependent  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/circular-reactive-dependency": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## 悪い

analyzer が追跡するリアクティブな参照の流れが提供元 A → 使用側 B → 提供元 A と循環しています。同名の無関係な変数ではなく、同一の参照として記録された構成です。

## 良い

B → A の流れをなくします。元の値は A が管理し、B は computed の読み取りや action の通知を使います。同じ参照を A に戻さなければ、追跡対象の循環がなくなります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[ファイル間ルール一覧](../cross-file.md)
