---
title: JSX と TSX
---

<!-- Generated translation; source: guide/jsx.md -->

# JSX と TSX

> **ステータス:**JSX/TSX は、コンパイラ、リンター、型チェッカー、LSP、およびフォーマッタにわたってカバーされています。
> 型認識チェックはオプトインのままなので、React `.tsx` ファイルが誤って Vue JSX として扱われることはありません。
> スタンドアロン `.jsx`/`.tsx` モジュールの HMR が、依然として統合の主なギャップとして残っています。

Vize は、`.vue` と**同じコンパイラ クレート**を介して `.jsx` および `.tsx` Vue コンポーネントをコンパイルします
単一ファイル コンポーネント — VDOM および Vapor バックエンド、Croquis セマンティック分析、Canon タイプ
チェック、Patina lint、Maestro 言語サーバー。個別の Babel パイプラインはありません。
ランタイム JSX ファクトリ シム: JSX コンポーネントは Vue レンダー関数 (または Vapor に直接渡されます)
テンプレート) ネイティブ コンパイラによる。

これは、`.tsx` Vue コンポーネントが同じ Rust ネイティブ コンパイル、同じ型チェックを受け、
SFC と同じエディター エクスペリエンス — `<template>` の代わりに型指定された関数として作成されるだけです。

## JSX/TSX の有効化

`.jsx` および `.tsx` ファイルは、Vize バンドラー プラグインを通じて自動的にルーティングされます。
オプトインフラグを設定してコンパイルします。すでに Vize バンドラー統合を使用しているプロジェクトは JSX/TSX を選択します
サポート:

- `@vizejs/vite-plugin`
- `@vizejs/unplugin` (ロールアップ / Webpack / esbuild)
- `@vizejs/rspack-plugin`
- `@vizejs/nuxt`

```ts
// vite.config.ts — nothing JSX-specific is required
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

内部では、プラグインはネイティブ/WASM `compileJsx` エントリ ポイント (から公開されている) を呼び出します。
`@vizejs/native` および `@vizejs/wasm`)、ソースを下げてレンダリング コードと任意のコードを返します。
抽出されたスコープ付き CSS。

## オーサリング API

Vize JSX/TSX コンポーネントは**型指定されたパラメーターを持つ単純な関数**です。マクロはありませんし、
一般的な場合の `defineComponent` ラッパー — 型は関数から直接読み取られます
署名が削除され、ランタイム出力から消去されます (コストゼロ)。

- **Props**は**入力された最初のパラメータ**です。
- **エミットとスロット**は**型指定された2番目のパラメータ**であり、Vizeが提供する`Ctx<Emits, Slots>`です。
  コンテキスト (`emit`、`slots`、および `attrs` を使用し、Vue のセットアップ コンテキストをミラーリングします)。
- **デフォルトのプロパティ値**は、パラメータ パターンの**デフォルトの構造化**から取得されます。
  コンパイラはそれらを構造化から抽出します。

```tsx
import { computed, ref } from "vue";

type CounterProps = {
  label: string;
  start?: number;
};

type CounterEmits = {
  change: [value: number];
};

const Counter = ({ label, start = 0 }: CounterProps, { emit }: Ctx<CounterEmits>) => {
  const count = ref(start);
  const doubled = computed(() => count.value * 2);

  const increment = () => {
    count.value += 1;
    emit("change", count.value);
  };

  return (
    <section class="counter">
      <p>
        {label}: {count.value}
      </p>
      <p>Double: {doubled.value}</p>
      <button type="button" onClick={increment}>
        Increment
      </button>
    </section>
  );
};
```

小道具のみのコンポーネントでは、2 番目のパラメータを完全に省略できます。

```tsx
const Hello = ({ name }: { name: string }) => <h1>Hello, {name}!</h1>;
```

デフォルト値は、分割デフォルトとして書き込まれます。個別の `props` オプションは必要ありません。

```tsx
const Badge = ({ count = 0 }: { count?: number }) => <span class="badge">{count}</span>;
```

コンポーネント名はバインディング (`const Counter = …`) または関数宣言から取得されます。
(`function Card() { … }`)、ご想像どおりです。それ以外はすべて React のような JSX — 要素
ネスト、フラグメント (`<>…</>`)、式の子、および `onClick` などのイベント プロパティ。唯一の
Vue 固有の追加は、[下記](#scoped-styles) で説明されている `<style scoped>` 要素です。

> 上記のタイプのみのオーサリング フォームは、サポートされている一般的なケースです。ランタイムを合成中 `props`
> メタデータ、および `defineComponent(() => () => vnode)` セットアップ フォームは、フォローアップが予定されています。

## サポートされている JSX サーフェス

コンパイラは、JSX を SFC テンプレートで使用されるのと同じ Relief IR に下げ、その IR を VDOM に送信します。
またはVaporバックエンド。これらのフォームは、JSX/TSX テスト マトリックスでカバーされています。

- フラグメントとネストされた要素
- コンポーネント タグ、メンバー式タグ、および組み込み HTML/SVG タグ
- 静的属性、動的 `prop={expr}` バインディング、ブール短縮プロパティ、およびスプレッド プロップ
- プロップ名にエンコードされた Vue スタイルのオプション修飾子を含むイベント ハンドラー
- `v-if`、`v-else-if`、`v-else`、`v-show`、カスタム `v-*` ディレクティブ、および `v-model`
- 式の子、論理 JSX ブランチ、三項 JSX ブランチ、および `.map(...)` リストのレンダリング
- オブジェクトの子またはレンダープロップの子として書き込まれたスロット
- TSX 構文: 型付きパラメータ、戻りアノテーション、汎用 JSX 呼び出し、キャスト、および非 null アサート
- `<style scoped>` 抽出; template-literal `${expr}` 補間は高度な拡張機能でサポートされています
  場合もありますが、通常は静的クラスと CSS 変数の方が明確です。

正規のリスト形式は慣用的な JSX です。

```tsx
import { computed, ref } from "vue";

type Todo = {
  id: string;
  title: string;
  done: boolean;
};

type TodoListProps = {
  todos: Todo[];
  initialActiveId?: string;
};

const TodoList = ({ todos, initialActiveId }: TodoListProps) => {
  const activeId = ref(initialActiveId ?? todos[0]?.id);
  const activeTodo = computed(() => todos.find((todo) => todo.id === activeId.value));

  return (
    <section class="todo-panel">
      <header>
        <h2>{activeTodo.value?.title ?? "Select a todo"}</h2>
      </header>

      <ul class="todo-list">
        {todos.map((todo, index) => (
          <li
            key={todo.id}
            class={{ done: todo.done, active: todo.id === activeId.value }}
            data-index={index}
          >
            <button type="button" onClick={() => (activeId.value = todo.id)}>
              <span>{todo.title}</span>
              {todo.id === activeId.value ? <strong>Active</strong> : <em>{index + 1}</em>}
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
};
```

`.map(...)` コールバック エイリアス (`todo`、`index`) は、生成された型チェッカーのスコープ内に保持され、
LSP 仮想 TypeScript なので、ホバー、完了、診断、名前変更が同じバインディングで動作します。
あなたが著者です。

## 出力モード: VDOM 対 蒸気

各コンポーネントは**仮想 DOM**出力 (Vue のデフォルトのレンダラー) または
[**蒸気**](https://blog.vuejs.org/posts/vue-vapor) 出力。デフォルトは構成によって選択されます。
個々のコンポーネントはそれをオーバーライドできます。

### デフォルト設定

`compiler.jsxMode` は、`.jsx`/`.tsx` コンポーネントのグローバルなデフォルト バックエンドを設定します。 `"vdom"`を受け入れます
または `"vapor"`、デフォルトは `"vdom"` です。

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    // Default every .jsx/.tsx component to Vapor output.
    jsxMode: "vapor",
  },
});
```

`jsxMode` は `compiler.vapor` から独立しています: `vapor` は `.vue` SFC の Vapor を切り替えますが、`jsxMode`
JSX/TSX のデフォルトのバックエンドを制御します。プロジェクトは、JSX をデフォルトで使用しながら、SFC を VDOM 上に維持できます。
蒸気、またはその逆。 Vite プラグインは、`jsxMode` をプラグイン オプションとして直接受け入れます。
共有設定をオーバーライドします。

### コンポーネントごとのディレクティブ

個々のコンポーネントは、`"use strict"` をミラーリングするディレクティブ プロローグでデフォルトをオーバーライドします。

```tsx
// Compiled to Vapor regardless of the configured default.
const Fast = () => {
  "use vue:vapor";
  return <div class="fast" />;
};

// Compiled to Virtual DOM regardless of the configured default.
const Classic = () => {
  "use vue:vdom";
  return <div class="classic" />;
};
```

各コンポーネントは独立してルーティングされるため、**単一のファイルで両方のバックエンドを混在させることができます**。

```tsx
// vize.config: { compiler: { jsxMode: "vapor" } }

// No directive -> takes the configured default (Vapor here).
export const Dashboard = () => <main>{/* ... */}</main>;

// Opts back into Virtual DOM just for this component.
export const LegacyWidget = () => {
  "use vue:vdom";
  return <aside>{/* ... */}</aside>;
};
```

### 優先順位

コンポーネントの出力モードは次の順序で解決されます。

1. コンポーネントごとの `"use vue:vapor"` / `"use vue:vdom"` ディレクティブ。
2. 設定からの `compiler.jsxMode` のデフォルト (またはプラグインの `jsxMode` オプション)。
3. 組み込みフォールバック、`"vdom"`。

### 診断

不正な形式または矛盾するディレクティブは、黙って無視されるのではなく、報告されます。

- `"use vue:"` で始まるが、既知のモードを指定していないディレクティブ (次のようなタイプミス)
  `"use vue:vdomx"`) はコンパイル エラーです。
- 1 つのコンポーネント内の 2 つの競合するモード ディレクティブ (`"use vue:vapor"` の後に `"use vue:vdom"`)
  診断されている。解決済みモードでは最初のディレクティブが引き続き優先されます。
- `"use strict"` などの関連のないプロローグはそのまま残されます。

## スコープ付きスタイル

- \*コンポーネント内の `<style scoped>` 要素\*\*は、SFC の JSX に相当します。
  `<style scoped>` ブロック。コンパイル時に抽出され、ランタイムとしてレンダリングされることはありません。
  vnode — その CSS は、生成された `data-v-<hash>` スコープ ID、そのスコープ属性でスコープ書き換えられます。
  はコンポーネントの他の要素に挿入され、書き換えられた CSS は
  Bundler プラグインの CSS パイプライン。これは VDOM と Vapor バックエンドの両方で機能し、両方とも
  特定のコンポーネントの同じスコープ ID。

慣用的に、`<style scoped>` 要素はマークアップの後に**最後**になります。これは SFC の要素と一致します。
`<template>` → `<style>` の順序ですが、コンパイラーはそれが出現する場所から抽出します。

```tsx
type CardProps = {
  title: string;
};

const Card = ({ title }: CardProps) => (
  <article class="card">
    <h2>{title}</h2>

    <style scoped>{`
      .card {
        border: 1px solid currentColor;
        padding: 12px;
      }
    `}</style>
  </article>
);
```

### 動的スタイル値

動的なスタイル設定には、通常のクラス バインディング、インライン スタイル オブジェクト、または CSS カスタム プロパティを優先します。
JSX/TSX。 `<style scoped>` 内のテンプレート リテラル補間 `${expr}` がサポートされており、
型チェックされていますが、これらはメインのオーサリング スタイルではなく、エスケープ ハッチです。

```tsx
type BoxProps = {
  color: string;
  gap: number;
};

const Box = ({ color, gap }: BoxProps) => (
  <section
    class="box"
    style={{
      "--box-color": color,
      "--box-gap": `${gap}px`,
    }}
  >
    <p>content</p>

    <style scoped>{`
      .box {
        color: var(--box-color);
        gap: var(--box-gap);
      }
    `}</style>
  </section>
);
```

`<style>` 要素**`scoped` なし**は、通常の要素として扱われ、そのままレンダリングされます。
抽出されていない。

`<style scoped>{`.box { カラー: ${color}; }`}</style>` も機能し、型チェッカーでカバーされます。
ただし、スコープ付きスタイルシートが実際にコンポーネント式を参照する必要がある場合に備えて保持してください。
SFC `<style>` ブロック内で使用されるリテラル CSS `v-bind(...)` 関数構文はサポートされていません。
JSX スタイル ブロック内のフォームを作成します。

## フォーマット

Glyph は、OXC パーサーとフォーマッタを使用して JSX/TSX スクリプト コンテンツをフォーマットします。 `.vue` ファイルでは、
`<script lang="jsx">`、`<script lang="tsx">`、および `<script setup lang="tsx">` は JSX/TSX として解析されます
プレーンな TypeScript にフォールバックする代わりに、JSX の子と TSX アノテーションは次のようにフォーマットされます。
実際の構文:

```vue
<script setup lang="tsx">
type CardProps = {
  title: string;
  items: string[];
};

const Card = ({ title, items }: CardProps) => (
  <section class="card">
    <h2>{title}</h2>
    {items.map((item) => (
      <span key={item}>{item}</span>
    ))}
  </section>
);
</script>
```

スタンドアロン `.jsx`/`.tsx` モジュールは、`.vue` ファイルとともに `vize fmt` によって検出され、フォーマットされます。
同じ JSX/TSX ソースタイプの処理を使用します。

```bash
# Formats .vue, .jsx, and .tsx files by default
vize fmt src --write
```

## 型チェック

JSX/TSX の型チェックは、`typeChecker.jsxTypecheck` による**オプトイン**であり、デフォルトは**`false`**です。
意図的にデフォルトではオフになっています。リポジトリには、使用してはいけない React `.tsx` ファイルが含まれている可能性があります。
Vue JSX として型チェックされます。

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  typeChecker: {
    enabled: true,
    jsxTypecheck: true,
  },
});
```

有効にすると、`vize check` は Canon を通じて `.jsx`/`.tsx` Vue コンポーネントの型チェックを行います。生成された
仮想ファイルは TSX ではなくプレーンな TypeScript であり、作成されたコンポーネント コントラクトを保持します。

- 型指定された最初のパラメータは props 型のままです。
- `Ctx<Emits, Slots>` はセットアップ本体と JSX 式に表示されたままになります。
- イベント ハンドラー、バインドされたプロパティ、`v-if`/`v-show`、カスタム ディレクティブ、スコープ スタイルの補間
  式が使用されると、通常の TypeScript 読み取りとして再発行されます。
- `v-model` ターゲットは書き込み可能な自己割り当てとして再発行されるため、読み取り専用または非左辺値バインディング
  結合時に診断されます。
- `.map(...)` リストの本体は生成されたコールバック内で再発行されるため、値/インデックスのエイリアスは保持されます。
  推論された要素タイプ。

診断は**元のソースの場所**(CLI の JSON として、および
LSP)、すべての意味のある仮想 TS 範囲が、作成したソース範囲にマッピングされるためです。

```tsx
type FieldProps = {
  model: {
    readonly value: string;
  };
};

const Field = ({ model }: FieldProps) => <input v-model={model.value} />;
```

上記の例では、割り当て対象として `model.value` がチェックされています。読み取り専用の場合、
診断は、生成されたコードではなく、TSX ソースの `model.value` に到達します。

```bash
# Type-check a project including its .jsx/.tsx Vue components.
# .jsx/.tsx files are collected only when typeChecker.jsxTypecheck is enabled.
vize check src
```

スタンドアロンの JSX/TSX コンポーネントは、チェック用のプレーンな仮想 TypeScript に下位にあります。を含むSFC
`<script lang="jsx">`、`<script lang="tsx">`、または一致する `script setup` ブロックは次のように具体化されます。
`.vue.tsx` 仮想ファイルなので、TypeScript はスクリプト ブロック内の JSX 構文を解析します。 LSP と CLI の共有
同じ低下であるため、Corsa 診断はエディターと
コマンドライン。

## エディター / LSP

`.jsx`/`.tsx` Vue コンポーネントを `vize lsp` をサポートするエディターで開くと、同じ言語が表示されます
SFC としての機能 —**SFC ラッパーは必要ありません**:

- 診断
- ホバー
- 完成
- 定義へ移動
- 参考文献
- 名前の変更
- 文書記号
- セマンティックトークン
- コードアクション
- `<style scoped>` ブロックの埋め込み CSS 診断

構造的機能 (ドキュメント シンボル、セマンティック トークン、スコープ スタイルの診断、コード アクション) が機能する
解析されたドキュメントから取得され、いつでも利用できます。タイプ認識機能 (診断、ホバー、
完了、定義へ移動、参照、名前変更) は、`typeChecker.jsxTypecheck` の場合にのみ到達します。
有効になっているため、React `.tsx` ファイルはエディターでも Vue JSX として扱われません。

## 糸くず

Vize の Patina lint ルールは、**OXC から直接投影されたゼロコスト ルール IR を通じて JSX/TSX 上で実行されます。
AST**。マークアップ指向のルールは合成 SFC テンプレートを再構築しません。 JSX 要素を読み取り、
属性を直接指定します。 `.map(...)` リスト キー チェックなど、Vue テンプレート シェイプを必要とするルールが実行されます。
低くなったレリーフツリーの上に。セマンティック ルールは、Croquis によってサポートされています。これは、
SFC。

これは、JSX/TSX lint が文字列の一部に依存せずに同じクラスの問題を捕捉することを意味します。
一致:

```tsx
const BrokenMedia = () => (
  <article>
    <img src="/avatar.png" />
    <button accessKey="s" autoFocus>
      Save
    </button>
  </article>
);
```

上の例は JSX ソースとして lint されています。

- `a11y/img-alt` は、欠落している `alt` を報告します。
- `a11y/no-access-key` は `accessKey` を報告します。
- `a11y/no-autofocus` は `autoFocus` をレポートします。

リストの主要なルールは、慣用的な JSX `.map(...)` 形状を理解します。

```tsx
const KeyedList = ({ rows }: { rows: Array<{ id: string; label: string }> }) => (
  <ul>
    {rows.map((row) => (
      <li key={row.id}>{row.label}</li>
    ))}
  </ul>
);
```

診断と修正は JSX ソース範囲にマップされるため、CLI 出力とエディターの装飾は JSX ソース範囲を指します。
変更する必要がある要素または小道具。

```bash
# Lint .vue, .html, .jsx, and .tsx files
vize lint src
```

lint および型チェック モデルについては、[静的解析](./static-analysis.md) を参照してください。
[ルール](../rules/index.md) 具体的なルールの出力。

## 制限事項

現在のエッジに注意してください。

- **型チェックはオプトインです。**`typeChecker.jsxTypecheck` はデフォルトでは `false` であるため、Vue/React が混在しています
  リポジトリが誤って React TSX を Vue JSX チェッカー経由でルーティングすることはありません。
- **HMR はまだ `.jsx`/`.tsx` モジュールに接続されていません。**JSX コンパイラーは現在、
  完全なコンポーネント オブジェクト モジュールではなくレンダリング関数モジュールなので、Vue HMR 境界はありません
  に取り付ける。完全なコンポーネント モジュール出力と状態保持 HMR は計画されたフォローアップです。まで
  その後、`.jsx`/`.tsx` コンポーネントへの編集は通常のリロードに戻ります。
- **JSX `<style scoped>` ブロック内のリテラル CSS `v-bind(...)` はサポートされていません。**`${expr}` を使用してください
  テンプレート リテラル補間。サポートされている型チェックされた形式です。

## も参照

- [構成](./configuration.md) — `compiler.jsxMode` キーと `typeChecker.jsxTypecheck` キー、
  さらに、完全な共有構成シェイプも含まれます。
- [Vite Plugin](./vite-plugin.md) — 推奨されるバンドラー統合。
- [静的分析](./static-analysis.md) — lint と型チェックがコンパイラ パイプラインを共有する方法。
- [`examples/jsx-tsx`](https://github.com/ubugeeei-prod/vize/tree/main/examples/jsx-tsx) —
  コンパイラ、リンター、型チェッカー、LSP、およびフォーマッタの範囲に焦点を当てた JSX/TSX ソースの例。
