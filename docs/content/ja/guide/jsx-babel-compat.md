---
title: Babel JSX 互換
---

# Babel JSX 互換

> **ステータス:** オプトインであり、デフォルトでは無効です。`compiler.jsxCompat` は設定ローダーが読み取り、
> native/WASM の `compileJsx` バインディングと Vize のバンドラープラグインが解釈します。

Vize は `.jsx` と `.tsx` を自前のコンパイラクレートでコンパイルします。そのため出力はテンプレートコンパイラと
同じ形、つまりブロックツリーであり、`v-if` / `v-for` は JavaScript から降ろされ、各ノードにパッチフラグが付き
ます。[`@vue/babel-plugin-jsx`](https://github.com/vuejs/babel-plugin-jsx) はそのいずれも行いません。素の
`createVNode` 呼び出しを出力し、ブロックを開かず、`&&`・`?:`・`.map()` はただの JavaScript のまま残し、
デフォルトではパッチフラグを一切出しません。

この違いの大部分は実行時には見えません。残りの部分こそ、このスイッチが存在する理由です。Babel プラグインから
移行するプロジェクトには、Vize のセマンティクスではなくプラグインのセマンティクスを要求する手段が必要です。
`compiler.jsxCompat: "babel"` がそのスイッチです。

このページが扱うのは**互換セマンティクス**です。オーサリング API、型の表面、Vapor/VDOM の出力切り替えについては
[JSX & TSX ガイド](./jsx.md)を参照してください。

## 有効にする

```json
{
  "compiler": {
    "jsxCompat": "babel"
  }
}
```

このキーは `"native"`（デフォルト）と `"babel"` を受け付けます。それ以外の値はビルドを失敗させるのではなく
`"native"` にフォールバックします。認識できない `jsxMode` の扱いと同じで、設定値の書き間違いがコンパイルを
止めてはならないためです。

同じ値は `compileJsx` バインディングが直接受け取ります。

```js
import { compileJsx } from "@vizejs/native";

const result = compileJsx(source, {
  filename: "App.tsx",
  lang: "tsx",
  jsxCompat: "babel",
});
```

`@vizejs/wasm` も同じ `jsxCompat` オプションを公開しています。Vite、unplugin、Rspack、Nuxt の各
エントリポイントは設定された `jsxCompat` を `compileJsx` へ渡し、各オプション型でも `jsxMode`、
`vapor` と並べて `jsxCompat` を直接指定できます。

## なぜオプトインでプロジェクト単位なのか

**デフォルトで無効。** `"native"` がデフォルトであり、そのままデフォルトであり続ける必要があります。これを
反転させると、Babel セマンティクスを求めていない既存のすべての Vize プロジェクトの出力が黙って変わってしまい
ます。

**プロジェクト単位で、コンポーネント単位の指定はない。** `jsxMode` は `"use vue:vapor"` /
`"use vue:vdom"` のプロローグでコンポーネントごとに選べます。VDOM と Vapor のコンポーネントは 1 つのモジュール
の中で問題なく共存でき、それぞれが独立したレンダー関数だからです。互換モードはそうではありません。互換モードは
**モジュール単位**の出力の形を変えます。Babel プラグインは JSX 式をその場で書き換えるので
`const A = () => <div />` は `const A = …` のまま残りますが、Vize は独立した `render` エクスポートを出力し
ます。1 つのファイルの半分だけを互換モードでコンパイルすると、互いに噛み合わない 2 つのモジュール形状が同じ
ファイルから出てしまいます。そのため互換モードはプロジェクトに対して一度だけ設定するものであり、ディレクティブ
プロローグの形式は意図的に用意していません。

## プラグインオプションの対応

Babel プラグイン自身のオプションには、Vize の設定ファイル上の綴りがありません。それぞれが
[`vize_atelier_jsx`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_jsx) クレートの
`compile_jsx_with_babel_*` エントリポイントの引数であり、`jsxCompat` が `"babel"` でない限りすべて何もしま
せん。

| `@vue/babel-plugin-jsx` | Vize のエントリポイント                     |
| ----------------------- | ------------------------------------------- |
| `transformOn`           | `BabelJsxOptions::transform_on`             |
| `pragma`                | `compile_jsx_with_babel_pragma`             |
| `mergeProps`            | `compile_jsx_with_babel_merge_props`        |
| `isCustomElement`       | `BabelJsxCustomizations::is_custom_element` |
| `enableObjectSlots`     | `compile_jsx_with_babel_object_slots`       |
| 任意の組み合わせ        | `compile_jsx_with_babel_customizations`     |

この表に載っていないプラグインオプションが 2 つあります。

- **`optimize`** に対応するものは Vize にはありません。Vize の出力は常に最適化されており、それはプラグインの
  `optimize: true` が生成するものと同じだからです。プラグインのデフォルトは `optimize: false` で、プラグイン
  自身の README も、有効にすると「特定の再レンダリングをスキップすることがある」と警告しています。つまり互換
  モードが埋めるべき差は*最適化しない*方向、すなわちパッチフラグを持たない出力です。
- **`resolveType`** は未実装です。下の「保留中のもの」を参照してください。

`enableObjectSlots` はプラグインでも Vize の互換レーンでもデフォルトが `true` です。コンポーネントの唯一の子
として渡された単独の識別子や呼び出し式は、すでにスロットオブジェクトである可能性があるため実行時に判定されます。
`false` を渡すと、その値は常にデフォルトスロットの生の子として扱われます。

## このモードが適用されない場所

**Vapor 出力。** `@vue/babel-plugin-jsx` は vdom 時代のプラグインで、定義している出力形状はすべて
`createVNode` ツリーであり、Vapor に対応するものはありません。したがって `jsxCompat: "babel"` と
`jsxMode: "vapor"` の組み合わせには定義された意味がなく、黙って無視するのではなく診断として拒否されます。

```text
compiler.jsxCompat: "babel" is not supported with Vapor output: @vue/babel-plugin-jsx has no
Vapor equivalent. Use jsxMode "vdom" for babel compatibility, or drop jsxCompat to use Vize's own
Vapor semantics.
```

**SSR 出力。** プラグインのオプションはクライアント側の vnode ツリーを記述するものです。そのため SSR
コンパイルでは Babel レーン全体、すなわち `transformOn` と `enableObjectSlots` のヘルパー、
`isCustomElement` の述語、`mergeProps: false`、および Babel 固有の降ろし処理をすべて適用せず、中途半端に
混ざった出力を出す代わりに Vize 自身の SSR セマンティクスを使います。

どちらも意図的な結論であり、蒸し返されないようクレート側に記録してあります。

## 保留中のもの

コーパスの 2 行は divergent ではなく `deferred` として記録されています。互換モードそのものではなく、無関係な
コンパイラ側の作業を待っているためです。

| 行                        | Babel の挙動                             | 待っているもの                                                                                                                                                                |
| ------------------------- | ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `options/resolve_type_on` | `{ props: { … }, name: "A" }` を付加する | 型駆動の props/emits 推論。[#1497](https://github.com/ubugeeei-prod/vize/issues/1497) / [#1502](https://github.com/ubugeeei-prod/vize/issues/1502) で追跡している型解決が必要 |
| `slots/dynamic_slot_name` | 計算キー `{ [n]: () => … }` を出力する   | 動的スロットの降ろし処理。Vize は現在、警告してスロットを捨てる                                                                                                               |

## 互換性の測り方

互換性は記憶からではなく、**実物のプラグイン**と突き合わせて測っています。コーパスはバージョンを固定した
`@vue/babel-plugin-jsx` でコンパイルされ、その出力はコミット済みの正解として記録され、Rust のテストスイートが
その記録を Vize の出力と並べてスナップショットし、行ごとに明示的な判定を付けます。

| 成果物                                                            | 役割                                           |
| ----------------------------------------------------------------- | ---------------------------------------------- |
| `crates/vize_atelier_jsx/tests/babel_compat/fixtures/corpus.json` | 入力と、各入力に与えるプラグインオプション     |
| `crates/vize_atelier_jsx/tests/babel_compat/oracle.mjs`           | コーパスを実物のプラグインに通す               |
| `crates/vize_atelier_jsx/tests/babel_compat_oracle.rs`            | Babel の出力を Vize の出力と並べて行ごとに記録 |
| `crates/vize_atelier_jsx/tests/BABEL_COMPAT_INVENTORY.md`         | 判定表を文章の形にしたもの、および合計         |

行ごとの判定、ほぼすべての行に共通する全体的な差異（モジュール形状、ブロックツリー、パッチフラグ、降ろされない
制御フロー）、そして現在の合計は、すべて
[`BABEL_COMPAT_INVENTORY.md`](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_atelier_jsx/tests/BABEL_COMPAT_INVENTORY.md)
にあります。この合計は `babel_compat_verdict_totals` のテストで固定されており、コーパスからずれることがあり
ません。このページが合計を一切引用しないのはそのためです。数値は元の場所で読んでください。

記録をローカルで再生成・検証するには次を実行します。

```bash
node crates/vize_atelier_jsx/tests/babel_compat/oracle.mjs --check
cargo test -p vize_atelier_jsx --test babel_compat_oracle
node --test tests/tooling/babel-jsx-oracle.test.ts
```

## 関連項目

- [JSX & TSX](./jsx.md) — オーサリング API、型付きの props と emits、スコープ付きスタイル、そして `jsxMode`。
- [設定](./configuration.md) — `compiler.*` の全キーと設定ファイルの探索順。
- [`examples/jsx-tsx`](https://github.com/ubugeeei-prod/vize/tree/main/examples/jsx-tsx) — 実行可能な JSX/TSX プロジェクト。
