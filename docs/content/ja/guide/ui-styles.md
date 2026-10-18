---
title: UI スタイル
---

<!-- Reviewed translation; source: guide/ui-styles.md; scope: complete document -->

# UI スタイル

[コンポーネントの使用例](./ui/index.md)で使いたいコンポーネントを選び、必要なスタイルだけを追加します。
下のプレビューは実際の Button の使用例です。**Style** で Paper、Signal、Atelier を切り替えられます。

<iframe src="/component-previews/app/index.html?family=button" title="Button スタイルの操作できるプレビュー" loading="lazy" width="100%" height="440" style="border:1px solid #8885;border-radius:8px"></iframe>

[スタイルのプレビューを開く](/component-previews/app/index.html?family=button)

## コンポーネントにスタイルを追加する

`@vizejs/ui` のコンポーネントは、見た目を整えるスタイルシートがなくても動作します。Vize が用意したスタイルを使う場合は、ベースの CSS、1つのパレット、ページで使う各コンポーネントの CSS をインポートします。

```ts
import "@vizejs/ui/base.css";
import "@vizejs/ui/theme-preset-paper.css";
import "@vizejs/ui/component-button.css";
import "@vizejs/ui/component-input.css";
import "@vizejs/ui/component-textarea.css";
import "@vizejs/ui/component-checkbox.css";
import "@vizejs/ui/component-switch.css";
import "@vizejs/ui/component-dialog.css";
import "@vizejs/ui/component-card.css";
import "@vizejs/ui/component-badge.css";
import "@vizejs/ui/component-alert.css";
import "@vizejs/ui/component-tooltip.css";
import "@vizejs/ui/component-tabs.css";
import "@vizejs/ui/component-breadcrumb.css";
import "@vizejs/ui/component-pagination.css";
import "@vizejs/ui/component-stepper.css";
// Add these only when their low-level behavior is used without the JS entry:
// import "@vizejs/ui/component-progress-bar.css";
// import "@vizejs/ui/component-scroll-area.css";
// import "@vizejs/ui/motion.css";
```

```vue
<script setup lang="ts">
import { Button } from "@vizejs/ui/button";
import "@vizejs/ui/base.css";
import "@vizejs/ui/theme-preset-paper.css";
import "@vizejs/ui/component-button.css";
</script>

<template>
  <main data-vize-theme="paper">
    <Button>Save changes</Button>
  </main>
</template>
```

この SFC は [Vite+ 連携](./vite-plus.md)を設定したプロジェクトで使えます。
全ページで読み込みたい CSS は、アプリケーションのエントリーにインポートしてください。

`base.css` はセマンティックトークン、密度、forced-colors ポリシーを提供します。これは既存の `theme.css` エクスポートのエイリアスです。アプリケーションの要素をリセットしたり、スタイルの付いていない Vize コンポーネントを変更したりすることはありません。コンポーネントファイルは素の CSS アセットであり、JavaScript や Vue の API は変わりません。あるコンポーネントをインポートしても、他のコンポーネントのビジュアルルールが追加されることはありません。

Input、Textarea、Checkbox、Switch にはそれぞれ専用の CSS エクスポートがあります。テキストフィールドはネイティブの編集とリサイズの挙動を保持します。Checkbox はチェック状態と混在状態を区別し、Switch は状態が変わるとつまみが移動します。短い状態遷移のアニメーションは、`prefers-reduced-motion` が有効な場合には停止します。キーボード操作時もフォーカスは表示され、forced-colors モードではネイティブのチェックボックスのマークが保たれます。コンポーネントファイルは各テーマ境界でローカルの上書きをリセットするため、Signal のページ内にネストされた Paper のフォームも Paper の書体とプロポーションを保ちます。

Card、Badge、Alert、Tooltip にもそれぞれ個別のビジュアルファイルがあります。Card は `variant`、`density`、`tone` のフックを使います。Badge はラベル、カウント、ステータステキストを区別します。Alert は閉じるボタンを追加せずにライブリージョンのバリアントに従います。Tooltip はトリガーのキーボードフォーカスを見えたままにし、フローティング位置が計測された後にのみ短い登場アニメーションを開始します。静的な Card の背景部分はアニメーションしません。Badge のトーン変化と Alert/Tooltip の登場アニメーションは、`prefers-reduced-motion` が有効な場合には停止し、forced-colors モードではシステムの境界線が復元されます。これらのスタイルは Shadow DOM ホストを含め、ネストされたテーマ境界でリセットされます。

Tabs、Breadcrumb、Pagination、Stepper にも、それぞれ独立した CSS ファイルがあります。
Tabs では、選択状態を示す線が短いアニメーションで移動し、選択中のパネルがなめらかに表示されます。Breadcrumb の区切り文字は、利用側のマークアップで指定します。
Pagination と Stepper は、スタイルを付けていない場合のナビゲーション動作を変えずに、現在の位置を表示します。画面幅が狭くラベルが収まらない場合、Stepper は横方向の並びを保ったままスクロールします。
`prefers-reduced-motion` ではアニメーションが停止し、forced-colors モードでもフォーカスと選択状態を見分けられます。Paper、Signal、Atelier のスタイルの上書きは、ネストされたテーマと Shadow DOM ホストの境界でリセットされます。

既存の ProgressBar と ScrollArea の構造およびモーションのレシピも、CSS のみの独立したファイルとして公開されています。それらの JavaScript エントリは、必要な挙動のためにすでにレガシーの集約スタイルシートを読み込みます。JavaScript エントリを使わずに CSS フックを使う場合や、スタイルシートを明示的に制御する場合は、独立したファイルをインポートしてください。1 つのページで両方の経路をインポートするのは避けてください。

| プリセット | 特徴                                                               |
| ---------- | ------------------------------------------------------------------ |
| `paper`    | 温かみのある紙、インク、細い罫線、角張ったコントロール             |
| `signal`   | シャープなエッジと控えめな奥行きを持つ、密度の高いグラファイトの面 |
| `atelier`  | 静かなスタジオ風のニュートラルカラーに、抑えたアクセントを 1 つ    |

スタイルを切り替えられるようにするには、上の1つのプリセットだけを読み込む記述を、選択肢にしたいプリセットのインポートに置き換えます。各スタイルシートは自身のスコープ内でのみ有効になります。

```ts
import "@vizejs/ui/theme-preset-paper.css";
import "@vizejs/ui/theme-preset-signal.css";
import "@vizejs/ui/theme-preset-atelier.css";

document.documentElement.dataset.vizeTheme = "signal";
```

既存の `midnight`、`play`、`high-contrast`、`headless` プリセットも引き続き利用できます。ダイアログやツールチップがドキュメントの body にテレポートされる場合は、フローティングコンテンツがページと同じパレットを継承するよう、`<html>` に `data-vize-theme` を設定してください。保存された設定を使う場合は、`@vizejs/ui/theme-scope` が描画前に実行されるブートストラップスクリプトを提供します。以前の `theme.css`、`theme-preset-*.css`、`style.css` のインポートも引き続き動作します。ただし `style.css` にはすでにベースとすべてのレガシープリセットが含まれているため、`base.css` と一緒にインポートするのは避けてください。

Button には短いホバー、押下、フォーカスのフィードバックがあります。`component-dialog.css` を使うと、Dialog は表示時にアニメーションし、200ms の退出アニメーションを行います。閉じると、フォーカスの閉じ込め、外側の inert、スクロールロックは即座に解除されます。退出中のシートはアニメーションが終わるまで inert のままで、支援技術からも隠されます。Headless の Dialog は引き続き即座にアンマウントされ、`prefers-reduced-motion: reduce` に一致する場合はスタイル付きの Dialog も同様です。どちらのビジュアルファイルも、forced-colors モードで明確な境界を保ちます。Vize のカスケードレイヤーの外にあるアプリケーションの CSS は、どのルールでも上書きできます。
