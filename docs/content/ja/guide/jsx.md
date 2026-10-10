---
title: JSX と TSX
---

<!-- Reviewed translation; source: guide/jsx.md -->

# JSX と TSX

> **対応状況:** JSX/TSX はコンパイラ、リンター、型チェッカー、LSP、フォーマッタで扱えます。
> 型情報を使う検査は明示的に有効にします。React の `.tsx` ファイルを誤って Vue JSX として検査しないためです。
> スタンドアロンの `.jsx`/`.tsx` モジュールでは、HMR 対応が主な未完了項目です。

Vize は `.jsx`/`.tsx` の Vue コンポーネントを、`.vue` ファイルと同じコンパイラクレートでコンパイルします。
VDOM/Vapor のコード生成、Croquis の解析、Canon の型チェック、Patina の lint、Maestro の言語サーバーを利用できます。
VDOM のモジュールでは、元の import、宣言、export、パラメータ、コンポーネント内のレキシカルな束縛を保持します。

## JSX/TSX の有効化

Vize のバンドラープラグインは `.jsx` と `.tsx` を自動的に処理します。コンパイル用の有効化フラグは不要です。
次のいずれかを導入済みなら、JSX/TSX のコンパイルも利用できます。

- `@vizejs/vite-plugin`
- `@vizejs/unplugin`（Rollup / webpack / esbuild）
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

プラグインは `@vizejs/native` または `@vizejs/wasm` の `compileJsx` を呼び出します。
この処理でソースを中間表現へ変換し、レンダー関数のコードと、抽出した scoped CSS を返します。

## オーサリング API

Vize の JSX/TSX コンポーネントは、**型付きパラメータを持つ通常の関数**です。
一般的なケースではマクロや `defineComponent` によるラップは不要です。
関数シグネチャから型を読み取り、実行時の出力から型注釈を消去するため、型のための実行時コストは発生しません。

- **props** は、型を付けた第 1 引数です。
- **emits と slots** は、第 2 引数の `Ctx<Emits, Slots>` で表します。Vize が提供するこのコンテキストには、Vue の setup コンテキストと同様に `emit`、`slots`、`attrs` があります。
- **props のデフォルト値**は、引数の分割代入に指定します。コンパイラがその値を取り出します。

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

props だけを受け取るコンポーネントでは、第 2 引数を省略できます。

```tsx
const Hello = ({ name }: { name: string }) => <h1>Hello, {name}!</h1>;
```

デフォルト値も分割代入で指定できるため、別途 `props` オプションを書く必要はありません。

```tsx
const Badge = ({ count = 0 }: { count?: number }) => <span class="badge">{count}</span>;
```

コンポーネント名は、変数の束縛（`const Counter = …`）または関数宣言（`function Card() { … }`）から取得します。
要素の入れ子、フラグメント（`<>…</>`）、子要素内の式、`onClick` などのイベント用 props は、React と同様の JSX 構文です。
Vue 固有の追加として、[後述の `<style scoped>`](#scoped-styles) があります。

> VDOM のブロック本体を持つコンポーネントでは、分割代入またはインラインのオブジェクト型から実行時の props 名を宣言します。
> 実行時バリデータ、名前付きの型からの推論、明示的な `defineComponent` の setup 形式は未対応です。
> ブロック内の setup 処理でレキシカルな `this`、`arguments`、`new.target` を使うと診断します。

<span id="サポートされている-jsx-サーフェス"></span>

## 対応する JSX 構文

JSX は SFC テンプレートと同じ Relief IR に変換され、VDOM または Vapor のバックエンドへ渡されます。
JSX/TSX のテストで確認している構文は次のとおりです。

- フラグメントと要素の入れ子
- コンポーネントタグ、メンバー式によるタグ、組み込みの HTML/SVG タグ
- 静的属性、動的な `prop={expr}`、値を省略した真偽値 props、スプレッド props
- イベントハンドラ。Vue のイベントオプション修飾子は props 名で指定できます。
- `v-if`、`v-else-if`、`v-else`、`v-show`、独自の `v-*` ディレクティブ、`v-model`
- 子要素内の式、論理演算子や三項演算子による分岐、`.map(...)` によるリスト描画
- 子要素のオブジェクトや render prop で記述したスロット
- 型付き引数、戻り値の型注釈、ジェネリックな JSX 呼び出し、キャスト、非 null アサーション
- `<style scoped>` の抽出。テンプレートリテラルの `${expr}` も扱えますが、通常は静的クラスと CSS 変数の方が読みやすくなります。

リストは、通常の JSX と同様に `.map(...)` で記述します。

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

`.map(...)` のコールバック引数（`todo`、`index`）は、型チェッカーと LSP 用の仮想 TypeScript でもスコープ内に保持します。
ホバー、補完、診断、名前変更は、元のソースにある同じ束縛を対象にします。

<span id="出力モード-vdom-対-蒸気"></span>

## 出力モード: VDOM と Vapor

コンポーネントは、Vue の標準レンダラーである **Virtual DOM** または [**Vapor**](https://blog.vuejs.org/posts/vue-vapor) 向けにコンパイルされます。
設定でデフォルトを選び、コンポーネントごとに上書きできます。

### デフォルト設定

`compiler.jsxMode` は `.jsx`/`.tsx` 全体のデフォルトを指定します。
値は `"vdom"` または `"vapor"` で、デフォルトは `"vdom"` です。

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

`jsxMode` と `compiler.vapor` は独立しています。`vapor` は `.vue` SFC を、`jsxMode` は JSX/TSX を対象にします。
SFC を VDOM、JSX を Vapor にする構成も、その逆も可能です。
Vite プラグインに直接指定した `jsxMode` は、共有設定より優先されます。

### コンポーネントごとのディレクティブ

`"use strict"` と同様に、関数の先頭へディレクティブを書いてデフォルトを上書きします。

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

各コンポーネントの出力先は独立しているため、**1 つのファイルで VDOM と Vapor を併用できます**。

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

出力モードは次の順で決まります。

1. コンポーネント先頭の `"use vue:vapor"` / `"use vue:vdom"`。
2. 共有設定の `compiler.jsxMode`、またはプラグインオプションの `jsxMode`。
3. 組み込みのデフォルト `"vdom"`。

### 診断

無効なディレクティブや指定の衝突は診断します。

- `"use vue:"` で始まり、既知のモードを指定していないものはコンパイルエラーです。例: `"use vue:vdomx"`。
- 1 つのコンポーネントで `"use vue:vapor"` と `"use vue:vdom"` を指定すると診断します。出力モードには先に書いた指定が使われます。
- `"use strict"` など、モードに関係しないディレクティブは保持します。

<a id="scoped-styles"></a>

## スコープ付きスタイル

コンポーネント内の `<style scoped>` は、SFC の同名のブロックに相当します。
コンパイル時に抽出するため、実行時の `<style>` vnode としては描画しません。
生成した `data-v-<hash>` スコープ ID で CSS を書き換え、他の要素にスコープ属性を付け、バンドラープラグインの CSS 処理へ渡します。
VDOM と Vapor の両方で利用でき、同じコンポーネントには同じスコープ ID が付きます。

通常は SFC の `<template>` → `<style>` と同じ順で、マークアップの**末尾**へ配置します。
ただし、コンパイラは記述位置にかかわらず抽出します。

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

動的な値には、クラスの束縛、インラインのスタイルオブジェクト、CSS カスタムプロパティを使うと読みやすくなります。
`<style scoped>` 内の `${expr}` によるテンプレートリテラル補間も型チェックできますが、必要な場合に使う補助手段です。

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

`scoped` を付けていない `<style>` は通常の要素として描画され、抽出されません。
``<style scoped>{`.box { color: ${color}; }`}</style>`` も型チェックの対象です。
scoped スタイルシートからコンポーネント内の式を参照する必要がある場合に使ってください。
SFC の `<style>` で使う CSS の `v-bind(...)` 構文は、JSX のスタイルブロックでは未対応です。

## フォーマット

Glyph は OXC のパーサーとフォーマッタで JSX/TSX を整形します。
`.vue` 内の `<script lang="jsx">`、`<script lang="tsx">`、`<script setup lang="tsx">` も JSX/TSX として解析するため、
JSX の子要素や TSX の型注釈を、通常の TypeScript と区別して整形します。

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

`vize fmt` は `.vue` に加えてスタンドアロンの `.jsx`/`.tsx` も検出し、同じ JSX/TSX の処理で整形します。

```bash
# Formats .vue, .jsx, and .tsx files by default
vize fmt src --write
```

## 型チェック

JSX/TSX の型チェックは、`typeChecker.jsxTypecheck` を明示的に有効にして使います。デフォルトは **`false`** です。
Vue と React が混在するリポジトリで、React の `.tsx` を Vue JSX として検査しないためです。

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

有効にすると、`vize check` は Canon で `.jsx`/`.tsx` の Vue コンポーネントを型チェックします。
生成する仮想ファイルは TSX ではなく通常の TypeScript で、次のように元のコンポーネントの型情報を保持します。

- 第 1 引数の型を props の型として保持します。
- `Ctx<Emits, Slots>` を setup 本体と JSX の式から参照できます。
- イベントハンドラ、props の束縛、`v-if`/`v-show`、独自ディレクティブ、scoped スタイル内の補間式は、TypeScript の読み取り式として出力します。
- `v-model` の代入先は書き込みの検査用に自己代入として出力し、readonly な値や左辺に置けない式を元の束縛位置で診断します。
- `.map(...)` の本体は生成したコールバック内へ出力し、値とインデックスの引数には推論した要素型を保持します。

仮想 TypeScript の意味のある範囲を元のソース範囲へ対応付けるため、CLI の JSON と LSP の診断はいずれも**元のソース位置**を指します。

```tsx
type FieldProps = {
  model: {
    readonly value: string;
  };
};

const Field = ({ model }: FieldProps) => <input v-model={model.value} />;
```

この例では `model.value` を代入先として検査します。readonly な場合は、生成コードではなく TSX ソース内の `model.value` を診断します。

```bash
# Type-check a project including its .jsx/.tsx Vue components.
# .jsx/.tsx files are collected only when typeChecker.jsxTypecheck is enabled.
vize check src
```

スタンドアロンの JSX/TSX は、型チェック用の通常の TypeScript へ変換します。
一方、SFC の `<script lang="jsx">`、`<script lang="tsx">`、対応する `script setup` は `.vue.tsx` の仮想ファイルにします。
これにより、TypeScript はスクリプトブロック内の JSX を解析できます。
LSP と CLI は同じ変換を使うため、Corsa の診断はエディターでもコマンドラインでも同じソース範囲を指します。

## エディター / LSP

`vize lsp` に対応したエディターでは、`.jsx`/`.tsx` の Vue コンポーネントに SFC と同じ機能を使えます。
**SFC でラップする必要はありません**。

- 診断
- ホバー
- 補完
- 定義への移動
- 参照の検索
- 名前変更
- ドキュメントシンボル
- セマンティックトークン
- コードアクション
- `<style scoped>` 内の CSS 診断

ドキュメントシンボル、セマンティックトークン、scoped スタイルの診断、コードアクションは、解析した文書から利用できます。
型情報を使う診断、ホバー、補完、定義への移動、参照、名前変更には `typeChecker.jsxTypecheck` の有効化が必要です。
エディターでも React の `.tsx` を誤って Vue JSX として扱わないようにしています。

<span id="糸くず"></span>

## lint

Patina の JSX/TSX 用 lint は、OXC AST から直接投影したルール用 IR を使います。
マークアップ用ルールは疑似的な SFC テンプレートを作らず、JSX の要素と属性を直接読み取ります。
`.map(...)` の key 検査など、Vue テンプレートの形が必要なルールは変換後の Relief ツリーを使います。
セマンティックなルールは、SFC と同じ Croquis の解析を利用します。

そのため、文字列の部分一致に頼らず、JSX/TSX の構文から次の問題を検出できます。

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

上の例では、次のルールが JSX のソースを診断します。

- `a11y/img-alt`: `alt` の指定漏れ。
- `a11y/no-access-key`: `accessKey` の使用。
- `a11y/no-autofocus`: `autoFocus` の使用。

リストの key を調べるルールも、通常の `.map(...)` による記述を扱えます。

```tsx
const KeyedList = ({ rows }: { rows: Array<{ id: string; label: string }> }) => (
  <ul>
    {rows.map((row) => (
      <li key={row.id}>{row.label}</li>
    ))}
  </ul>
);
```

診断と修正は JSX のソース範囲へ対応付けられます。CLI の出力とエディターの表示は、修正する要素や props を指します。

```bash
# Lint .vue, .html, .jsx, and .tsx files
vize lint src
```

lint と型チェックの仕組みは[静的解析](./static-analysis.md)を、具体的なルール出力は[ルール一覧](../rules/index.md)を参照してください。

## 制限事項

現在の制限は次のとおりです。

- **型チェックは明示的に有効にします。** `typeChecker.jsxTypecheck` のデフォルトは `false` で、Vue/React が混在するリポジトリで誤って React TSX を検査しないためです。
- **`.jsx`/`.tsx` モジュールの HMR は未対応です。** VDOM は元の import、export、ブロック内の setup 状態、JSX の引数のデフォルト値とスコープを保持します。ただし、バンドラーが Vue の HMR 境界を登録しないため、編集時は通常の再読み込みになります。
- **Vapor と SSR で元のモジュールを保持する処理は未完了です。** スタンドアロンのレンダラーが保持できない import、export、setup 文、捕捉したローカルの束縛は診断します。静的なスタンドアロンのレンダラーとコンポーネント単位の出力は利用できます。
- **実行時ヘルパーの別名は隠せません。** `_openBlock` などの生成するヘルパーと元の束縛が衝突すると、不正なモジュールを出力せず診断します。
- **JSX の `<style scoped>` 内では CSS の `v-bind(...)` は未対応です。** 型チェックできる `${expr}` によるテンプレートリテラル補間を使ってください。

<span id="も参照"></span>

## 関連項目

- [設定](./configuration.md): `compiler.jsxMode`、`typeChecker.jsxTypecheck`、共有設定全体。
- [Vite プラグイン](./vite-plugin.md): 推奨するバンドラー連携。
- [静的解析](./static-analysis.md): lint と型チェックで共有するコンパイラの処理。
- [`examples/jsx-tsx`](https://github.com/ubugeeei-prod/vize/tree/main/examples/jsx-tsx): コンパイラ、リンター、型チェッカー、LSP、フォーマッタの確認用 JSX/TSX 例。
