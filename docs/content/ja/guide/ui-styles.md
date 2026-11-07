---
title: UI スタイル
description: ネイティブ CSS、セマンティックなセレクタ、公開デザイントークンで @vizejs/ui に任意のビジュアルを追加します。
---

<!-- Reviewed translation; source: guide/ui-styles.md; scope: complete document -->

# UI スタイル

`@vizejs/ui` は、アクセシブルな振る舞いとビジュアルデザインを分離しています。CSS なしで始める、用意されたスタイルを追加する、同じセマンティック属性を自分のデザインシステムから使う、という選択ができます。Tailwind は必須でも特別扱いでもありません。

<iframe src="/component-previews/app/index.html?family=button" title="Button のすべてのビジュアルと操作状態を比較する" loading="lazy" width="100%" height="560" style="border:1px solid #8885;border-radius:12px"></iframe>

[スタイルラボを全画面で開く](/component-previews/app/index.html?family=button)

## 3 つの import から始める

共有トークン、1 つのプリセット、この画面で使うコンポーネントのスタイルだけを読み込みます。

```ts
import "@vizejs/ui/base.css";
import "@vizejs/ui/theme-preset-paper.css";
import "@vizejs/ui/component-button.css";
```

```vue
<script setup lang="ts">
import { Button } from "@vizejs/ui/button";
</script>

<template>
  <main data-vize-theme="paper">
    <Button type="button" variant="solid" tone="accent">変更を保存</Button>
    <Button type="button" variant="outline" tone="neutral">プレビュー</Button>
    <Button type="button" variant="quiet" tone="danger">削除</Button>
  </main>
</template>
```

すべてのルートで共有する CSS はアプリケーションのエントリーから読み込んでください。コンポーネントの import が、テーマや別のコンポーネントのビジュアルルールを暗黙に追加することはありません。

## パレットではなく、デザインの声を選ぶ

各プリセットは、配色だけでなく形状、タイポグラフィ、奥行き、密度、モーションまで異なります。型付き Props とセマンティックな DOM 契約はすべて共通です。

| プリセット      | デザインの方向性                                             | 最初の選択に向く用途             |
| --------------- | ------------------------------------------------------------ | -------------------------------- |
| `atelier`       | 静かなニュートラル、柔らかな形状、抑制した奥行き             | 生産性ツール、編集ツール         |
| `paper`         | 温かいインク、細い罫線、角張った操作部、ほぼ浮かない表現     | 出版、情報密度の高い UI          |
| `signal`        | コンパクトなグラファイト、鋭い輪郭、等幅書体、浅い奥行き     | 運用画面、開発者ツール           |
| `midnight`      | 暗い発光面と、制御されたレイヤー状の影                       | メディア、集中用ワークスペース   |
| `play`          | 丸い形状、弾むような奥行き、表情のある押下フィードバック     | ソーシャル、コンシューマー製品   |
| `high-contrast` | 厳格な形状、強い状態分離、装飾的な奥行きなし                 | 状態の判別しやすさを最優先する UI |
| `headless`      | 固有の見た目を持たないセマンティックトークン境界             | 完全に独自のデザインシステム     |

1 つのプリセット import で製品の声を固定できます。利用者が切り替えられる製品では、複数を読み込みます。

```ts
import "@vizejs/ui/theme-preset-atelier.css";
import "@vizejs/ui/theme-preset-midnight.css";
import "@vizejs/ui/theme-preset-paper.css";

document.documentElement.dataset.vizeTheme = "midnight";
```

各スタイルシートは、一致する `data-vize-theme` の範囲でだけ有効です。Shadow DOM ホストを含む、テーマの入れ子にも対応します。

```vue
<template>
  <main data-vize-theme="atelier">
    <EditorWorkspace />

    <aside data-vize-theme="signal">
      <OperationsPanel />
    </aside>
  </main>
</template>
```

Dialog や Tooltip などが `document.body` に Teleport される場合は、`<html>` にテーマを設定します。保存済みの設定を初回描画より前に適用する場合は `@vizejs/ui/theme-scope` を使います。

## 公開カスタムプロパティで調整する

ビジュアル CSS は `data-vize-ui` から始まり、その内側に型付き状態をネストします。追従が必要な BEM クラスや、生成されたユーティリティ文字列はありません。

```css
.checkout-actions {
  --vize-ui-button-radius: 0.5rem;
  --vize-ui-button-shadow: none;
  --vize-ui-button-shadow-hover: none;
  --vize-ui-button-hover-lift: 0px;
  --vize-ui-button-press-depth: 1px;

  [data-vize-ui="button"] {
    letter-spacing: 0.01em;

    &[data-tone="danger"] {
      --vize-ui-button-fill: var(--checkout-danger);
    }
  }
}
```

Button が公開するコンポーネントプロパティは次のとおりです。

| プロパティ                        | 用途                                   |
| --------------------------------- | -------------------------------------- |
| `--vize-ui-button-fill`           | 塗りとアウトラインの色源               |
| `--vize-ui-button-ink`            | 塗りの上の前景色                       |
| `--vize-ui-button-radius`         | コントロールの形状                     |
| `--vize-ui-button-shadow`         | 通常時の奥行き                         |
| `--vize-ui-button-shadow-hover`   | ホバー時の奥行き                       |
| `--vize-ui-button-hover-lift`     | ホバー時の移動量                       |
| `--vize-ui-button-press-depth`    | 押下時の移動量                         |
| `--vize-ui-button-press-scale`    | 押下時の縮尺                           |
| `--vize-ui-button-hover-strength` | 塗りつぶしのホバー色の混合率           |
| `--vize-ui-button-tint-strength`  | outline と quiet のホバー色の混合率    |

Vize のカスケードレイヤー外にあるアプリケーション CSS は通常どおり優先されます。CSS Modules、scoped style、Tailwind、UnoCSS、グローバル CSS のどれからでも、コンポーネントを変えずに同じ契約を利用できます。

パッケージのソースはネイティブ CSS Nesting で記述されています。ビルド時に Lightning CSS が対象ブラウザー向けへ変換するため、アプリケーションは通常の CSS を受け取り、Vize のツールチェーンを必要としません。

## アクセシビリティのポリシーを保つ

`vize.policy` レイヤーはビジュアルプリセットより後に置かれ、次の操作要件を守ります。

- `:focus-visible` でキーボードフォーカスを見える状態に保つ
- 粗いポインターでは最小 44 CSS ピクセルの操作領域を確保する
- `prefers-reduced-motion` では装飾的な動きとローディングの回転を止める
- forced-colors では背景、境界、文字、フォーカス、無効状態をシステム色へ対応させる
- ホバー専用フィードバックは、実際にホバーできる端末でだけ動かす

このポリシーを上書きする場合は、同等の操作性を保ってください。色のコントラスト、アクセシブルな名前、正確なローディング文言は、引き続きアプリケーション側の責任です。

## ルートで使う分だけ読み込む

よく使うビジュアルアセットは独立して公開されています。

```ts
import "@vizejs/ui/component-input.css";
import "@vizejs/ui/component-checkbox.css";
import "@vizejs/ui/component-switch.css";
import "@vizejs/ui/component-tabs.css";
import "@vizejs/ui/component-dialog.css";
import "@vizejs/ui/component-card.css";
import "@vizejs/ui/component-alert.css";
import "@vizejs/ui/component-tooltip.css";
```

Input、Textarea、Checkbox、Switch、Button、Dialog、Card、Badge、Alert、Tooltip、Tabs、Breadcrumb、Pagination、Stepper は、それぞれコンポーネント単位のアセットを持ちます。ProgressBar と ScrollArea にも、低レベルで明示的に使うための CSS-only アセットがあります。同じスタイルを含む単体アセットとレガシーの集約エントリーを同時に読み込まないでください。

`style.css` はベースと従来のプリセットを含む互換用の集約ファイルです。新しいコードでは明示的な import を推奨します。`style.css` と `base.css` は併用しないでください。

## フレームワークとレンダラー

ビジュアル層は静的 CSS です。setup コードを実行せず、ブラウザー API を読み取らず、サーバーのマークアップも変更しません。

- Nuxt では CSS エクスポートを `nuxt.config.ts` の `css` に追加するか、アプリケーションのエントリーから import します。[Nuxt](../integrations/nuxt.md)を参照してください。
- Void ではページまたはアプリケーションのエントリーから import し、ルートの CSS グラフに含めます。[Void](../integrations/void.md)を参照してください。
- DOM、SSR、ハイドレーション、Vapor は同じセマンティック属性を使います。Vapor 専用のスタイルシートは不要です。[Vapor](./vapor.md)を参照してください。

次は[コンポーネントカタログ](./ui/index.md)を開いてください。各リファレンスページには、操作できる使用例、コピー可能な SFC、型付き API、設計ガイド、規範的な振る舞い表、宣言済みの品質ゲートがあります。
