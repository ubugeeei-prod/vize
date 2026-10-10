---
title: Babel JSX 互換
---

<!-- Reviewed translation; source: guide/jsx-babel-compat.md -->

# Babel JSX 互換

> **対応状況:** 明示的に有効にする機能で、デフォルトでは無効です。
> 設定ローダーが `compiler.jsxCompat` を読み取り、native/WASM の `compileJsx` とバンドラープラグインへ渡します。

Vize は `.jsx`/`.tsx` を独自のコンパイラクレートで処理します。
標準の出力はテンプレートコンパイラと同様にブロックツリーを作り、JavaScript の分岐やリストを `v-if`/`v-for` 相当へ変換し、パッチフラグを付けます。
一方、[`@vue/babel-plugin-jsx`](https://github.com/vuejs/babel-plugin-jsx) は `createVNode` を出力し、ブロックを開きません。
`&&`、`?:`、`.map()` は通常の JavaScript として残り、デフォルトではパッチフラグも付けません。

多くの違いは実行結果に表れませんが、移行時に Babel プラグインと同じ意味で処理する必要がある場合には、`compiler.jsxCompat: "babel"` を指定します。
このページでは**互換モードの意味と制限**を説明します。
コンポーネントの書き方、型、Vapor/VDOM の選択は [JSX と TSX](./jsx.md)を参照してください。

## 有効にする

```json
{
  "compiler": {
    "jsxCompat": "babel"
  }
}
```

指定できる値は `"native"`（デフォルト）と `"babel"` です。
認識できない値は、`jsxMode` と同様にビルドを失敗させず `"native"` へフォールバックします。
`compileJsx` に直接渡すこともできます。

```js
import { compileJsx } from "@vizejs/native";

const result = compileJsx(source, {
  filename: "App.tsx",
  lang: "tsx",
  jsxCompat: "babel",
});
```

`@vizejs/wasm` も同じ `jsxCompat` オプションを公開しています。
Vite、unplugin、Rspack、Nuxt は設定値を `compileJsx` へ渡します。
各プラグインのオプション型でも、`jsxMode`、`vapor` と同様に `jsxCompat` を直接指定できます。

## なぜオプトインでプロジェクト単位なのか

**既存プロジェクトの出力を維持するため、デフォルトは `"native"` です。**
Babel 互換を求めていないプロジェクトの出力を、設定なしで変更しないようにしています。

**互換モードはプロジェクト単位で指定します。**
`jsxMode` とは異なり、コンポーネント先頭のディレクティブによる指定はありません。
`jsxMode` はコンポーネントごとの VDOM/Vapor 出力を選びますが、`jsxCompat` はモジュール全体で Babel の処理方式を使うかどうかを選びます。
現在の VDOM 出力は、元の宣言、export、レキシカルなスコープを保持します。
ただし、Babel と Vize のブロック構造、制御フローの変換、パッチフラグには違いがあり、元のモジュールを保持するだけで移行の互換性が証明されるわけではありません。

## プラグインオプションの対応

Babel プラグイン固有のオプションを、Vize の設定ファイルへ同じ名前で指定することはできません。
これらは [`vize_atelier_jsx`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_jsx) の `compile_jsx_with_babel_*` 関数へ渡す引数です。
`jsxCompat` が `"babel"` のときにだけ適用します。

| `@vue/babel-plugin-jsx` | Vize のエントリポイント |
| --- | --- |
| `transformOn` | `BabelJsxOptions::transform_on` |
| `pragma` | `compile_jsx_with_babel_pragma` |
| `mergeProps` | `compile_jsx_with_babel_merge_props` |
| `isCustomElement` | `BabelJsxCustomizations::is_custom_element` |
| `enableObjectSlots` | `compile_jsx_with_babel_object_slots` |
| 任意の組み合わせ | `compile_jsx_with_babel_customizations` |

次の 2 つは、この表とは別に扱います。

- **`optimize`** に相当する設定はありません。Vize の標準出力は最適化されており、Babel の `optimize: true` に近い形です。Babel のデフォルトは `optimize: false` なので、互換モードではパッチフラグのない出力など、この違いを扱います。Babel の README も、最適化を有効にすると一部の再描画を省略する可能性を説明しています。
- **`resolveType`** は未実装です。[保留中の項目](#保留中のもの)を参照してください。

`enableObjectSlots` のデフォルトは、Babel と Vize の互換モードの両方で `true` です。
コンポーネントの唯一の子が識別子や呼び出し式の場合、その値がスロットオブジェクトかどうかを実行時に調べます。
`false` にすると、その値は常にデフォルトスロットの通常の子として扱います。

## このモードが適用されない場所

**Vapor 出力では利用できません。**
Babel プラグインが定義する出力は `createVNode` のツリーで、Vapor に相当する処理はありません。
`jsxCompat: "babel"` と `jsxMode: "vapor"` を併用すると、無視するのではなく診断して拒否します。

```text
compiler.jsxCompat: "babel" is not supported with Vapor output: @vue/babel-plugin-jsx has no
Vapor equivalent. Use jsxMode "vdom" for babel compatibility, or drop jsxCompat to use Vize's own
Vapor semantics.
```

**SSR では Vize 本来の処理を使います。**
Babel のオプションはクライアント側の vnode ツリーを対象にしています。
SSR コンパイルには、`transformOn` と `enableObjectSlots` のヘルパー、`isCustomElement` の判定、`mergeProps: false`、その他の Babel 固有の変換を適用しません。
一部だけを混在させず、Vize の SSR 処理を使う方針です。これらの制限はクレート側にも記録しています。

<span id="保留中のもの"></span>

## 保留中の項目

現在のコーパスでは、次の 1 項目が `deferred` です。
互換モード自体ではなく、型解決の実装を待っています。

| 行 | Babel の挙動 | 未完了の依存項目 |
| --- | --- | --- |
| `options/resolve_type_on` | `{ props: { … }, name: "A" }` を付加する | 型からの props/emits 推論。[#1497](https://github.com/ubugeeei-prod/vize/issues/1497) / [#1502](https://github.com/ubugeeei-prod/vize/issues/1502) の型解決に依存する |

`slots/dynamic_slot_name` は現在 `equivalent` です。
計算したキー `{ [n]: () => … }` によるスロットは、保留項目には含まれません。

## 互換性の測り方

互換性は、バージョンを固定した**実際の Babel プラグイン**と比較します。
入力コーパスをプラグインでコンパイルし、その出力を正解データとしてコミットします。
Rust のテストは、そのデータと Vize の出力を並べて保存し、入力ごとに明示的な判定を付けます。

| 成果物 | 役割 |
| --- | --- |
| `crates/vize_atelier_jsx/tests/babel_compat/fixtures/corpus.json` | 入力と、それぞれに適用するプラグインオプション |
| `crates/vize_atelier_jsx/tests/babel_compat/oracle.mjs` | 実際のプラグインによるコーパスの処理 |
| `crates/vize_atelier_jsx/tests/babel_compat_oracle.rs` | Babel と Vize の出力を入力ごとに記録するテスト |
| `crates/vize_atelier_jsx/tests/BABEL_COMPAT_INVENTORY.md` | 入力ごとの判定と合計を説明する一覧 |

詳しい判定、モジュールの形・ブロック構造・パッチフラグ・制御フローの変換に関する差異、現在の合計は、
[`BABEL_COMPAT_INVENTORY.md`](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_atelier_jsx/tests/BABEL_COMPAT_INVENTORY%2Emd)にあります。
合計は `babel_compat_verdict_totals` のテストでコーパスと一致することを確認するため、このページでは重複して掲載しません。
数値は元の一覧を参照してください。

ローカルで正解データを再生成・検証するには、次を実行します。

```bash
node crates/vize_atelier_jsx/tests/babel_compat/oracle.mjs --check
cargo test -p vize_atelier_jsx --test babel_compat_oracle
node --test tests/tooling/babel-jsx-oracle.test.ts
```

## 関連項目

- [JSX と TSX](./jsx.md): コンポーネントの書き方、型付き props と emits、スコープ付きスタイル、`jsxMode`。
- [設定](./configuration.md): `compiler.*` のキーと設定ファイルの探索順。
- [`examples/jsx-tsx`](https://github.com/ubugeeei-prod/vize/tree/main/examples/jsx-tsx): 実行可能な JSX/TSX プロジェクト。
