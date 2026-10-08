---
title: ファイル間の複雑さ
---

<!-- Generated translation; source: guide/cross-file-complexity.md -->

# ファイル間の複雑さ

Vize は、保持した式の AST、テンプレートの制御領域、解決済みのプロジェクト情報から複雑度を計算します。
ソース中の記号の個数や実行時間を測るものではありません。レポートには、目的が異なる 3 つの値があります。

| 見方                                | 答える問い                                                                           | 利用先                                 |
| ----------------------------------- | ------------------------------------------------------------------------------------ | -------------------------------------- |
| **自身のテンプレート (own)**        | このコンポーネントのテンプレートには判断がいくつあり、どれほど深くネストしているか   | `vue/max-template-complexity`          |
| **描画するテンプレート (rendered)** | 子コンポーネントを重複なくたどると、どれほどのテンプレート複雑度に届くか             | Doctor のホットスポット notice         |
| **重み付きプロジェクトスコア**      | 解析対象のテンプレートとコンポーネント境界には、どれほどの制御やデータの流れがあるか | ファイル間レポートとホットスポット順位 |

## 自身のテンプレートの計算

解析するテンプレートは **サイクロマティック複雑度 1、認知的複雑度 0** から始まります。
サイクロマティック複雑度は `1 + 判断の数`、認知的複雑度は次の加点の合計です。
単純な `<p>Hello</p>` も **1 / 0** になります。script 内の分岐はテンプレートスコアに含めません。

現在のネストの深さを `d` とします。分岐、ループ、スコープ付きスロットの本体に入ると 1 段深くなります。
通常の要素やコンポーネントでは深くなりません。三項演算子では条件と両方の選択肢を調べる間も 1 段深くなるため、
条件内にある三項演算子にもネストの加点があります。

| 構文                           | サイクロマティック加点    | 認知的加点                   | 対応するソース           |
| ------------------------------ | ------------------------- | ---------------------------- | ------------------------ |
| `v-if`                         | 1                         | `1 + d`                      | 条件                     |
| 各 `v-else-if`                 | 1                         | `d` によらず 1               | 条件                     |
| `v-else`                       | 0                         | `d` によらず 1               | 分岐                     |
| `v-for`                        | 1                         | `1 + d`                      | コレクションの式         |
| 各 `?:`                        | 1                         | `1 + d`                      | 三項演算子の式           |
| 論理演算子の木                 | `&&`・`\|\|`・`??` の個数 | 同じ演算子の連続区間の数     | 論理演算子の木全体       |
| 引数を持つスコープ付きスロット | 0                         | 0。本体は深さ `d + 1`        | スロットのバインディング |
| 不明な式                       | 0                         | 0。別途 unknown の数を増やす | 式                       |

条件とループのコレクション式は、本体に入る前の深さで評価します。スコープ付きスロットの所有要素に付いた
バインディングも元の深さで評価し、子だけがスロットのネストを受けます。論理演算子の連続区間は、
深い分岐の中でも認知的加点が常に 1 です。

独立した `v-if` が 3 つあれば **4 / 3** です。同じ 3 条件を入れ子にすると、サイクロマティックは **4** のまま、
認知的は **1 + 2 + 3 = 6** になります。`templateMaxNesting` は、式のネストではなく、
**空でないテンプレート領域**の最大深さです。`{{ a ? b : c ? d : e }}` は **3 / 3** ですが、
テンプレートの最大ネストは **0** です。

### 論理演算子の連続区間と AST の境界

論理演算子の木は、直接つながった論理式からなります。括弧は境界になりません。演算子をソース順に読み、
種類が変わると連続区間が終わります。関数呼び出し、メンバーアクセス、TypeScript のラッパー、三項演算子は
現在の木の境界になり、その内側の論理式は別の木として計算します。次の例にはテンプレート側のネストがありません。

| 補間の式             | 演算子数         | 認知的加点           | 初期値 1 を含む own スコア |
| -------------------- | ---------------- | -------------------- | -------------------------- |
| `a && b && c`        | 2                | 連続区間 1           | 3 / 1                      |
| `a && b \|\| c && d` | 3                | 連続区間 3           | 4 / 3                      |
| `(a \|\| b) ?? c`    | 2                | 連続区間 2           | 3 / 2                      |
| `a && f(b \|\| c)`   | 2 つの木で合計 2 | 合計 2               | 3 / 2                      |
| `a ? b : c ? d : e`  | 三項演算子 2     | 三項演算子の `1 + 2` | 3 / 3                      |

### 評価する位置、除外、不明な式

補間、条件、コレクション式、バインディングの値と動的な名前、保持したイベントハンドラーの式、
ディレクティブの値と動的な引数、`.sync`、`v-memo`、`v-show`、`v-html`、`v-text`、動的なスロット名を調べます。
`v-model` は読み取りの式を 1 回だけ数えます。書き込み側は同じソースを指すため、重複して数えません。

テキスト、コメント、静的属性、ループの別名、スロット引数の束縛パターンは判断を増やしません。
通常のタグのネスト、`v-show` 自体、オプショナルチェーン (`?.`)、論理代入 (`&&=`・`||=`・`??=`)、
style 内の `v-bind()`、`v-once`、`v-cloak` も加点しません。ただし、除外する構文の中にある式は調べます。
例えば `v-show="ready && visible"` は **2 / 1** です。

スロット出口の fallback は、**fallback が存在するだけでは**判断もネストも増やしません。
中身は元の深さで調べるため、`<slot><p v-if="ready">Fallback</p></slot>` は **2 / 1** です。
親が記述したスロット内容は親の own に含み、子コンポーネントの実装は含みません。

保持した JS AST がない不透明な式、別言語の式、Vue 2 のフィルターチェーンは、それぞれ加点 0 の
`unknown` 行になります。文字列から複雑さを推測しません。現在のハンドラー本体のキャリアは、この式解析では
走査せず、unknown 行も増やしません。保持したハンドラー式は計算します。元のヘッドだけを保持したループは、
コレクション AST を走査せず、ループの構造だけを数えます。低いスコアだけでは未対応のコードが単純とは判断できません。
SFC の lint ルールは外部テンプレートと HTML 以外のテンプレート言語を対象にしません。

## Bad / Good の全加点を追う

**`vue/max-template-complexity` に対する Bad:** 次のテンプレートの own は、サイクロマティック **13**、
認知的 **25** です。この例は複雑度ルールに絞り、参照するデータやコンポーネントはアプリが用意するものとします。

```vue
<template>
  <section>
    <h1>{{ user ? user.name : "Guest" }}</h1>
    <DataTable :rows="rows">
      <template #cell="{ row, column }">
        <span v-if="column.key === 'status'" :class="row.active ? 'on' : 'off'">
          {{ row.status ?? "unknown" }}
        </span>
        <a v-else-if="column.key === 'link' && row.url" :href="row.url">{{ row.label }}</a>
        <template v-else>
          <em v-for="tag in row.tags" :key="tag">
            <b v-if="tag.pinned || tag.starred">{{ tag.hot ? "!" : "" }}</b>
          </em>
        </template>
      </template>
    </DataTable>
    <p v-if="!rows.length && !loading">No data</p>
  </section>
</template>
```

スコープ付きスロット自体は加点しませんが、本体の深さを 1 にします。ソース順の全加点は次のとおりです。

| 加点対象                                       | 深さ | サイクロマティック | 認知的 |
| ---------------------------------------------- | ---: | -----------------: | -----: |
| 初期経路                                       |    — |                  1 |      0 |
| `user ? user.name : 'Guest'`                   |    0 |                  1 |      1 |
| `v-if="column.key === 'status'"`               |    1 |                  1 |      2 |
| `row.active ? 'on' : 'off'`                    |    2 |                  1 |      3 |
| `row.status ?? 'unknown'`                      |    2 |                  1 |      1 |
| `v-else-if="column.key === 'link' && row.url"` |    1 |                  1 |      1 |
| この条件の `&&`                                |    1 |                  1 |      1 |
| `v-else`                                       |    1 |                  0 |      1 |
| `v-for="tag in row.tags"`                      |    2 |                  1 |      3 |
| `v-if="tag.pinned \|\| tag.starred"`           |    3 |                  1 |      4 |
| この条件の `\|\|`                              |    3 |                  1 |      1 |
| `tag.hot ? '!' : ''`                           |    4 |                  1 |      5 |
| `v-if="!rows.length && !loading"`              |    0 |                  1 |      1 |
| この条件の `&&`                                |    0 |                  1 |      1 |
| **合計**                                       |      |             **13** | **25** |

**このルールに対する Good:** 行ごとの分岐を子へ移すと、親のテンプレートを小さくできます。

```vue
<template>
  <RowList v-if="ready" :rows="rows" />
</template>
```

own は **2 / 1** です。初期経路の 1 に `v-if` の判断 1 を足し、認知的にはネストしていない分岐の 1 を足します。
`RowList` の実装は別に評価します。親の own が下がるのは、判断が親の記述から移った場合です。
同じ複雑なスロット内容を親に残せば、その加点は残ります。rendered や重み付きプロジェクトスコアには、
切り出した処理が引き続き含まれることがあります。

## Rendered: 到達する子を重複なく数える

ルートの own から始め、解決済みの**コンポーネント使用**エッジをたどります。到達するファイルの識別子を集合にし、
各コンポーネントの own を 1 回だけ足します。ほかの import エッジは描画エッジではありません。
同じタグの繰り返し、別々の分岐、ループの反復回数では加点を倍増させません。テンプレート情報がないファイルには
推測のスコアを足しませんが、`renderedComponents` はそのような到達先も数えるため、解析の網羅性は保証しません。

import によって `Page → Grid`、`Page → Card`、`Grid → Card` が解決されるとします。
次のテンプレートは、既存の rendered 複雑度フィクスチャの計算を示します。

```vue
<!-- Page.vue -->
<template>
  <main>
    <Card v-if="hero" :item="hero" />
    <Grid :items="items" />
    <Card :item="footer" />
  </main>
</template>
```

```vue
<!-- Grid.vue -->
<template>
  <ul>
    <li v-for="item in items" :key="item.id"><Card :item="item" /></li>
  </ul>
</template>
```

```vue
<!-- Card.vue -->
<template>
  <article v-if="item.visible">
    <h2>{{ item.title ?? "Untitled" }}</h2>
  </article>
</template>
```

| コンポーネント | Own サイクロマティック / 認知的 | 重複を除いた到達先 | Rendered サイクロマティック / 認知的  |
| -------------- | ------------------------------- | ------------------ | ------------------------------------- |
| Card           | 3 / 2                           | なし               | 3 / 2                                 |
| Grid           | 2 / 1                           | Card               | `2 + 3` / `1 + 2` = **5 / 3**         |
| Page           | 2 / 1                           | Grid、Card         | `2 + 2 + 3` / `1 + 1 + 2` = **7 / 4** |

`Page.renderedComponents` は 2 です。Page が Card を 2 回使い、Grid も Card を使いますが、
Card の初期経路を含むスコアは 1 回だけ足します。到達先が自身だけの自己再帰なら rendered = own で、
ほかのコンポーネント数は 0 です。own が各 **2 / 1** の相互再帰 `Ping ↔ Pong` では、どちらのルートも
rendered **4 / 2**、ほかのコンポーネント数 1、`recursive: true` になります。
再帰フラグはルート自身に戻るときだけ立ちます。到達先の別の循環だけでは、ルートを再帰とは扱いません。

## 重み付きプロジェクトスコア: 7 つの指標

`complexityReport.cyclomaticScore` と `cognitiveScore` は、解析対象のテンプレートの **own** を合計します。
rendered の合計ではありません。`dimensions` はこの合計と次の情報を組み合わせます。
`totalScore` は 7 指標の合計であり、平均や描画経路の数ではありません。

| 指標フィールド        | `complexityReport.input` のフィールドを使う計算式                                                                |
| --------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `templateControlFlow` | `templateCyclomatic + templateCognitive`                                                                         |
| `slotUsage`           | `2 × slotCount + 2 × templateScopedSlotCount`                                                                    |
| `propDrilling`        | `3 × propDrillingEdgeCount`                                                                                      |
| `globalState`         | `2 × globalStateReferenceCount`                                                                                  |
| `provideInject`       | `2 × max(provideInjectMaxDepth − 1, 0) + provideInjectReferenceCount + 2 × max(provideInjectFanoutCount − 1, 0)` |
| `fallthroughAttrs`    | `4 × fallthroughRiskCount`                                                                                       |
| `reactiveGraph`       | `reactiveNodeCount + 2 × reactiveEdgeCount + 10 × reactiveCycleCount`                                            |

入力は解析器のカウンターです。名前だけでは分かりにくい範囲は次のとおりです。

- `slotCount` は宣言されたスロットとコンポーネント使用に記録されたスロットを足します。
  スコープ付きスロット領域は `templateScopedSlotCount` でも加点します。実行時の呼び出し回数ではありません。
- `propDrillingEdgeCount` はコンポーネント使用ごとに渡す prop の項目数です。1 段だけの場合も数え、
  長い props の受け渡し経路だけを対象にするものではありません。
- `globalStateReferenceCount` は既存の `ShouldUseStoreToRefs` と `StoreDestructured` の検出数です。
  store への全アクセスやすべてのグローバル変数を数えるわけではありません。
- provide/inject ツリーの要約があれば、深さは最大深さ、参照数は provide と inject の合計です。
  fan-out は子の分岐数と provider の consumer 数の大きい方です。要約がなければ参照数はマッチ数になり、
  深さと fan-out は 0 のままです。
- `fallthroughRiskCount` は問題の可能性があるコンポーネント数と、危険な未消費の継承属性数を足します。
  要約がなければ、fallthrough 情報が問題の可能性を示すコンポーネントを数えます。
- リアクティブノード数はリアクティブソースの登録数で、同名の登録も数えます。エッジと循環はファイルごとの
  effect graph の要約から取り、任意の import や provide/inject のエッジは使いません。

`componentCount` は登録された Vue コンポーネント数で、独立した重みはありません。
`templateUnknown` は unknown 行の合計、`templateMaxNesting` はテンプレート領域の最大深さで、直接は加点しません。
カウンターの変換、加算、重み付けは整数の上限で飽和させます。スコアは桁あふれで巻き戻らず、
**4,294,967,295** が上限です。

### 数値を代入する完全な例

既存のスコア計算テストは次の入力を使います。レポート入力の計算例であり、特定の Vue コードだけから
これらすべてのプロジェクト情報が得られると主張する例ではありません。

```json
{
  "componentCount": 1,
  "templateCyclomatic": 6,
  "templateCognitive": 9,
  "templateUnknown": 1,
  "templateMaxNesting": 3,
  "templateScopedSlotCount": 4,
  "slotCount": 3,
  "propDrillingEdgeCount": 2,
  "globalStateReferenceCount": 4,
  "provideInjectMaxDepth": 3,
  "provideInjectReferenceCount": 5,
  "provideInjectFanoutCount": 4,
  "fallthroughRiskCount": 2,
  "reactiveNodeCount": 6,
  "reactiveEdgeCount": 7,
  "reactiveCycleCount": 1
}
```

| 指標               | 代入した式                      |   点数 |
| ------------------ | ------------------------------- | -----: |
| テンプレート制御   | `6 + 9`                         |     15 |
| スロット           | `2 × 3 + 2 × 4`                 |     14 |
| Props の受け渡し   | `3 × 2`                         |      6 |
| グローバル状態     | `2 × 4`                         |      8 |
| Provide/inject     | `2 × (3 − 1) + 5 + 2 × (4 − 1)` |     15 |
| 継承属性           | `4 × 2`                         |      8 |
| リアクティブグラフ | `6 + 2 × 7 + 10 × 1`            |     30 |
| **合計**           | `15 + 14 + 6 + 8 + 15 + 8 + 30` | **96** |

区分は 0–14 が `low`、15–34 が `moderate`、35–69 が `high`、70 以上が `extreme` です。
この例は **extreme** で、30 点の `reactive-graph` が最大の指標です。重み付きホットスポットも、
コンポーネントごとの入力に同じ式を適用し、合計の降順、次にファイル名で並べます。
ホットスポットの合計はプロジェクトの合計と一致するとは限りません。provide/inject の深さや fan-out は
個別に割り当てられ、それらの計算には非線形な部分があるためです。

## しきい値、診断位置、lint の有効化

own のルールは **サイクロマティック 11 または認知的 16 を超える**と警告します。同値は許容します。
どのプリセットでも有効にはならず、自動修正もルール固有のしきい値オプションもありません。
この定数は **2026-09-22** に測定した 40,724 テンプレートの p95 です。
当時のコーパスの分布が、現在の各プロジェクトの分布と同じとは限りません。

[Vite+ の設定](/ja/guide/vite-plus)で有効にし、`vp run lint` を実行します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      rules: {
        "vue/max-template-complexity": "warn",
      },
    },
  },
});
```

警告は開始 `<template>` タグを指します。加点 0 の行を除き、認知的加点、サイクロマティック加点の大きい順、
次にソース位置の早い順で最大 5 件を選び、選んだラベルをソース順に表示します。
Bad の例では status の `v-if`、class の三項演算子、`v-for`、tag の `v-if`、内側の三項演算子です。
要素全体ではなく条件や式の範囲が、実際に加点された処理を示します。

`templateComplexity` の加点対象には、ファイル先頭からのバイト位置 `start` / `end` と、
1 始まりの文字単位の `line` / `column` があります。コンポーネントは rendered の認知的複雑度の降順、
サイクロマティックの降順、ファイル名で並びます。Doctor は rendered が **106 または 139 を超える**と、
`VIZE_DOCTOR_TEMPLATE_COMPLEXITY_HOTSPOT` を **notice** として出します。主位置は最初の own 加点対象で、
それがなければバイト 0 の空範囲です。根拠には上位 3 件までの own 加点対象を示します。
ワークスペース内のパスで表せない場合は報告しません。rendered の上限と重み付きスコアの区分は独立しています。

WASM のファイル間解析結果では、`complexityReport`、`complexityHotspots`、`templateComplexity` を公開します。
重み付きレポートは探索用の指標で、それ自体でビルドを失敗させません。
明示的に有効化した lint ルールは設定した重大度の方針に従います。
不明・取得できない情報は解析範囲の限界であり、理解しやすいコンポーネントだという根拠にはなりません。

## 実装と既存の検証

- [指標の仕様と当時のコーパス](https://github.com/ubugeeei-prod/vize/blob/main/docs/davinci/plan/complexity-metrics.md)
- [L2 の領域と式の計算](https://github.com/ubugeeei-prod/vize/tree/main/davinci/vize_l1_to_l2/src/pass/cfg)
- [Lint の完全な例と診断ラベル](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/max_template_complexity/tests.rs)
- [ファイル間の重み、カウンター、到達性](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_croquis_cf/src/rules/complexity)
- [重み付きスコアの検証](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/complexity_tests.rs)
