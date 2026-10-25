---
title: 安定したスクリーンショットとアクセシビリティの確認
---

<!-- Reviewed translation; source: guide/musea-capture.md -->

<span id="stable-screenshots-and-accessibility-details"></span>

# 安定したスクリーンショットとアクセシビリティの確認

アニメーションの途中のフレームや点滅するキャレットによって画像の比較結果が変わるとき、
またはアクセシビリティの診断で問題のある要素を詳しく調べたいときに、このページを使ってください。
まず [Musea ガイド](./musea.md)に従ってギャラリーを起動し、撮影設定とレポートを確認してから、
比較の基準となる画像を更新します。

Musea は、スクリーンショットの撮影時に CSS アニメーションを停止し、テキスト入力のキャレットを隠します。
どちらもデフォルトの動作です。有限のアニメーションは最後まで進み、無限のアニメーションは最初の状態に戻ります。
以前の基準画像にアニメーションの途中のフレームが含まれていた場合、画像の差分が生じることがあります。

撮影時の動作は、Musea プラグインの設定で指定します。

```ts
musea({
  vrt: {
    capture: {
      animations: "disabled", // "allow" keeps animations running
      reducedMotion: "reduce", // default: "no-preference"
      caret: "hide", // "initial" keeps the text caret
    },
  },
});
```

操作後の画面を比較する場合は、`capture.beforeCapture` に非同期コールバックを指定します。
分離された Playwright の `Page` と、読み取り専用の `{ artPath, variantName, viewport }` を受け取ります。
CLI は Vite 設定からこの関数を読み込みます。`MuseaVrtRunner` にも同じ設定を渡せます。

```ts
musea({
  vrt: {
    capture: {
      async beforeCapture(page, { variantName }) {
        if (variantName !== "Submitted") return;
        await page.getByRole("textbox", { name: "Message" }).fill("Hello");
        await page.getByRole("button", { name: "Submit" }).click();
        await page.getByText("Message received", { exact: true }).waitFor();
      },
    },
  },
});
```

プレビューの準備と `waitSelector` の完了後に関数を実行して待機し、その後で `settleTime`、
要素の非表示、マスク、撮影設定を適用します。`workers` による並列実行や再試行でも、
撮影ごとに新しいページを使います。関数が例外または拒否を返すと、`beforeCapture failed:` で始まる
撮影エラーになり、ページとコンテキストを閉じます。失敗した撮影の今回の画像や基準画像は
作成・更新しません。

この例は、アニメーションの停止とキャレットの非表示を維持したまま、
ページに動きを抑える設定を要求します。`reducedMotion` に `"reduce"` を指定すると、
ブラウザーからページにこの設定を伝えます。実際の動きはページ側の対応によって変わります。
デフォルトは `"no-preference"` で、
アニメーションを停止する設定とは別に扱われます。

最初の撮影では、ギャラリーのサーバーを起動したまま、
[画像を比較する手順](./musea.md#visual-regression-testing)に従って実行してください。
基準画像、今回の画像、差分画像を比べ、コンポーネントの変更と撮影設定の変更のどちらが原因かを確認します。
意図した変更であることを確かめてから、新しい基準画像を承認してください。

アクセシビリティのレポートを JSON 形式で保存するには、`musea-vrt --a11y --json` を実行します。
CLI は出力先のディレクトリにレポートを書き込みます。デフォルトの出力先は `.vize` です。
`--json` を指定した場合は `a11y-report.json`、指定しない場合は `a11y-report.html` を確認してください。

それぞれの違反には、問題のある要素の件数を示す `nodes` と、要素ごとの詳細を示す `targets` が含まれます。
詳細には、セレクターの配列、HTML の抜粋、失敗の概要、`any`・`all`・`none` のチェック結果が保持されます。
axe が測定値を提供する場合、チェックの `data` にはコントラストの色と比率も含まれます。

目視確認が必要な結果の詳細は `incompleteResults` に保持されます。`incomplete` は件数を表します。
これらの結果は、確認済みの合格として扱わず、実際の要素と報告された根拠を確認してください。
HTML レポートで要素の詳細を開くと、マークアップ、失敗の理由、測定されたチェックのデータを調べられます。

監査エラーは、そのバリアントを監査できなかったことを意味します。
違反の件数だけで結論を出さず、まずエラーの内容を確認してください。
