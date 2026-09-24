---
title: UI スタイル
---

<!-- Generated translation; source: guide/ui-styles.md -->

# UI スタイル

`@vizejs/ui` のコンポーネントは、ビジュアル用のスタイルシートがなくても動作します。Vize のオプションのスタイルを使うには、ベース、パレットを 1 つ、そしてページで使うコンポーネントをインポートします。

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
// Add these only when their low-level behavior is used without the JS entry:
// import "@vizejs/ui/component-progress-bar.css";
// import "@vizejs/ui/component-scroll-area.css";
// import "@vizejs/ui/motion.css";
```

```vue
<template>
  <main data-vize-theme="paper">
    <Button>Save changes</Button>
  </main>
</template>
```

`base.css` はセマンティックトークン、密度、forced-colors ポリシーを提供します。これは既存の `theme.css` エクスポートのエイリアスです。アプリケーションの要素をリセットしたり、スタイルの付いていない Vize コンポーネントを変更したりすることはありません。コンポーネントファイルは素の CSS アセットであり、JavaScript や Vue の API は変わりません。あるコンポーネントをインポートしても、他のコンポーネントのビジュアルルールが追加されることはありません。

Input、Textarea、Checkbox、Switch にはそれぞれ専用の CSS エクスポートがあります。テキストフィールドはネイティブの編集とリサイズの挙動を保持します。Checkbox はチェック状態と混在状態を区別し、Switch は状態が変わるとつまみが移動します。小さな状態遷移は `prefers-reduced-motion` の下では停止します。キーボードユーザーにはフォーカスが見えたままで、forced-colors モードではネイティブのチェックボックスのマークが保たれます。コンポーネントファイルは各テーマ境界でローカルの上書きをリセットするため、Signal のページ内にネストされた Paper のフォームも Paper の書体とプロポーションを保ちます。

Card、Badge、Alert、Tooltip にもそれぞれ個別のビジュアルファイルがあります。Card は `variant`、`density`、`tone` のフックを使います。Badge はラベル、カウント、ステータステキストを区別します。Alert は閉じるボタンを追加せずにライブリージョンのバリアントに従います。Tooltip はトリガーのキーボードフォーカスを見えたままにし、フローティング位置が計測された後にのみ短い登場アニメーションを開始します。静的な Card のサーフェスはアニメーションしません。Badge のトーン変化と Alert/Tooltip の登場アニメーションは `prefers-reduced-motion` の下では停止し、forced-colors モードではシステムの境界線が復元されます。これらのスタイルは Shadow DOM ホストを含め、ネストされたテーマ境界でリセットされます。

既存の ProgressBar と ScrollArea の構造およびモーションのレシピも、CSS のみの独立したファイルとして公開されています。それらの JavaScript エントリは、必要な挙動のためにすでにレガシーの集約スタイルシートを読み込みます。JavaScript エントリを使わずに CSS フックを使う場合や、スタイルシートを明示的に制御する場合は、独立したファイルをインポートしてください。1 つのページで両方の経路をインポートするのは避けてください。

| プリセット | 特徴                                                               |
| ---------- | ------------------------------------------------------------------ |
| `paper`    | 温かみのある紙、インク、細い罫線、角張ったコントロール             |
| `signal`   | シャープなエッジと控えめな奥行きを持つ、密度の高いグラファイトの面 |
| `atelier`  | 静かなスタジオ風のニュートラルカラーに、抑えたアクセントを 1 つ    |

スタイルの切り替えを提供するには、上の単一のプリセットのインポートを、提供したいプリセットに置き換えます。各スタイルシートは自身のスコープ内でのみ有効になります。

```ts
import "@vizejs/ui/theme-preset-paper.css";
import "@vizejs/ui/theme-preset-signal.css";
import "@vizejs/ui/theme-preset-atelier.css";

document.documentElement.dataset.vizeTheme = "signal";
```

既存の `midnight`、`play`、`high-contrast`、`headless` プリセットも引き続き利用できます。ダイアログやツールチップがドキュメントの body にテレポートされる場合は、フローティングコンテンツがページと同じパレットを継承するよう、`<html>` に `data-vize-theme` を設定してください。保存された設定を使う場合は、`@vizejs/ui/theme-scope` が描画前に実行されるブートストラップスクリプトを提供します。以前の `theme.css`、`theme-preset-*.css`、`style.css` のインポートも引き続き動作します。ただし `style.css` にはすでにベースとすべてのレガシープリセットが含まれているため、`base.css` と一緒にインポートするのは避けてください。

Button には短いホバー、押下、フォーカスのフィードバックがあります。`dialog.css` を使うと、Dialog は表示時にアニメーションし、200ms の退出アニメーションを行います。閉じると、フォーカスの閉じ込め、外側の inert、スクロールロックは即座に解除されます。退出中のシートはアニメーションが終わるまで inert のままで、支援技術からも隠されます。Headless の Dialog は引き続き即座にアンマウントされ、`prefers-reduced-motion: reduce` に一致する場合はスタイル付きの Dialog も同様です。どちらのビジュアルファイルも、forced-colors モードで明確な境界を保ちます。Vize のカスケードレイヤーの外にあるアプリケーションの CSS は、どのルールでも上書きできます。
