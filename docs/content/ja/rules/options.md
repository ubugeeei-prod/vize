---
title: ルールのオプション
---
<!-- Reviewed translation; source: rules/options.md -->

# ルールのオプション
<!-- Type-aware reference: [型ルールのオプション](./options-type.md) -->

ルールの判定方法をプロジェクトに合わせたいときに、このページを使ってください。
Vite+ では `vite.config.ts` の `vue.linter.ruleOptions` に指定し、`vp run lint` で検査します。
単独で使う互換設定では、同じ内容を `linter.ruleOptions` に指定します。

`linter.ruleOptions` には、オプションを受け取るルールの設定をプロジェクトごとに指定します。
定義されていないフィールドを指定するとエラーになります。同じルールに対して、後から一致する設定がある場合は、
そのルールのオプション全体を置き換えます。重大度は `linter.rules` で別に設定し、
`ruleOptions` では有効なルールの判定方法を指定します。以下で明記する場合を除き、オプションだけではルールは有効になりません。

ルールと重大度の指定方法は、[ネイティブ Lint の実行ガイド](../guide/lint-execution.md)で確認できます。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  "lint": {
    "vize": {
      "rules": {
        "script/no-restricted-globals": "error",
        "vue/html-self-closing": "warn",
        "musea/prefer-design-tokens": "warn"
      },
      "ruleOptions": {
        "script/no-restricted-globals": {
          "globals": [
            {
              "name": "process",
              "message": "Read env via a typed helper."
            },
            {
              "name": "alert"
            }
          ]
        },
        "vue/html-self-closing": {
          "html": {
            "void": "always",
            "normal": "never",
            "component": "always"
          },
          "svg": "always",
          "math": "always"
        },
        "musea/prefer-design-tokens": {
          "tokens": [
            {
              "path": "color.primary",
              "value": "#3b82f6",
              "tier": "semantic"
            }
          ]
        }
      }
    }
  }
});
```

| ルール | オプションの形式 | 既定値と動作 |
| --- | --- | --- |
| `html/no-empty-palpable-content` | `{ contentDirectives?: string[] }` | 既定は空のリストです。目に見える内容を挿入するディレクティブの名前を、接頭辞を除いて完全一致で指定します。ルールは別に有効にしてください。 |
| `script/no-restricted-globals` | `{ globals?: Array<{ name: string; message?: string }> }` | オプションを省略すると、組み込みの禁止リストにある `process`、`localStorage`、`sessionStorage` を検査します。空でない `globals` を指定すると、組み込みのリストを置き換えます。 |
| `script/no-restricted-members` | `{ members?: Array<{ object: string; property: string; message?: string }> }` | `members` を設定し、ルールを有効にした場合だけ報告します。`message` を省略すると、共通の説明を表示します。 |
| `vue/valid-v-on` | `{ modifiers?: string[] }` | 既定は空の一覧です。大文字・小文字を区別する独自の名前を組み込みの修飾子に追加します。ルールを有効にしたり、ハンドラーの省略を許可したりする設定ではありません。範囲別の設定は継承した一覧を置き換え、`[]` で解除します。[修飾子の詳しい例](./reference/vue-valid-v-on.md)を参照してください。 |
| `vue/component-name-in-template-casing` | `{ casing?: "PascalCase" \| "kebab-case" }` | 既定は `PascalCase` です。登録済みのコンポーネントだけを対象にし、追加のグローバル名は既定で空です。 |
| `script/custom-event-name-casing` | `{ casing?: "camelCase" \| "kebab-case" }` | 既定は `camelCase` です。 |
| `script/define-props-destructuring` | `{ destructure?: "only-when-assigned" \| "always" \| "never" }` | 既定は `only-when-assigned` です。`always` は代入しない呼び出しも検査し、`never` は props オブジェクトでの参照を要求します。Vue 3.5 の分割代入はリアクティビティを保ちます。 |
| `vue/no-mutating-props` | `{ shallowOnly?: boolean }` | 既定は `false` です。`true` では、プロパティ自体の再代入を禁止したまま、入れ子のプロパティの変更を許可します。 |
| `vue/require-component-registration` | `{ globals?: string[] }` | アプリケーションのプラグインやプレビューのセットアップで登録するコンポーネント名を指定します。PascalCase と kebab-case の表記を許可します。ルールは別に有効にしてください。 |
| `vue/sfc-element-order` | `{ order?: Array<string \| string[]> }` | 既定は `[["script", "template"], "style"]` です。入れ子の配列は、その位置に配列内のどのブロックが現れてもよいことを表します。 |
| `vue/html-self-closing` | `{ html?: { void?: Style; normal?: Style; component?: Style }; svg?: Style; math?: Style }`。`Style` は `"always"`、`"never"`、`"any"` のいずれか | 既定は `html.void: "always"`、`html.normal: "any"`、`html.component: "always"`、`svg: "always"`、`math: "always"` です。 |
| `vue/v-on-event-hyphenation` | `"always" \| "never"` | コンポーネントの静的なイベントリスナー名を、ハイフン区切りにするかを指定します。 |
| `vue/attribute-hyphenation` | `"always" \| "never"` | テンプレートのコンポーネントに渡すプロパティの属性名を、ハイフン区切りにするかを指定します。 |
| `vue/valid-v-slot` | `{ allowModifiers?: boolean }` | 既定は `false`。`true` では、名前付きスロットの修飾子を許可します。引数なしの修飾子や、そのスロット自身のプロパティに依存する動的な名前は報告します。 |
| `musea/prefer-design-tokens` | `{ tokens?: Array<{ path: string; value: string; tier?: string }> }` | トークンのデータを指定し、ルールを有効にするか、空でないトークンのリストで暗黙に選択した場合だけ報告します。`tier` の既定は `primitive` です。 |
| `type/strict-boolean-expressions` | `{ allowString?: boolean; allowNumber?: boolean; allowNullableObject?: boolean; allowNullableBoolean?: boolean; allowNullableString?: boolean; allowNullableNumber?: boolean; allowNullableEnum?: boolean; allowAny?: boolean }` | `allowString`、`allowNumber`、`allowNullableObject` の既定は `true`、その他は `false` です。ルールを明示的に有効にする必要があります。 |

## `html/no-empty-palpable-content`

カスタムディレクティブは、フォーカスの移動、スタイルの適用、内容の挿入などに使われます。
プロジェクトで、目に見える内容を挿入すると確認できるディレクティブだけを指定してください。
`v-` を除いたテンプレートの名前に完全一致します。引数や修飾子は、その名前を変えません。
空のリストを指定すると既定の動作に戻り、ファイルごとの設定はオプション全体を置き換えます。
この設定はサニタイズの安全性を保証せず、ルールを有効にするものでもありません。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  "lint": {
    "vize": {
      "rules": {
        "html/no-empty-palpable-content": "error"
      },
      "ruleOptions": {
        "html/no-empty-palpable-content": {
          "contentDirectives": [
            "safe-html"
          ]
        }
      }
    }
  }
});
```

悪い（内容の挿入元を指定していない場合）:

```vue
<p v-sanitize="message" />
```

良い（内容を挿入するディレクティブを指定した場合）:

```vue
<p v-safe-html="message" />
```

<span id="scoped-entries"></span>

## ファイルごとの設定

`entries` がファイルに一致すると、その `linter.ruleOptions` が共通設定より優先されます。
同じルールのオプションは全体を置き換えます。`globals`、`members`、`order`、`tokens`
などの配列は、複数の設定から結合されません。

## `script/no-restricted-globals`

実行環境のグローバル変数へのアクセスを、プロジェクトで管理するラッパーに統一したいときに使います。
オプションを省略すると、`process`、`localStorage`、`sessionStorage` への参照を報告します。
空でない `globals` は組み込みの禁止リストを置き換えます。空のリストでは組み込みのリストに戻るため、
設定ミスでルールが意図せず無効になることはありません。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  "lint": {
    "vize": {
      "rules": {
        "script/no-restricted-globals": "error"
      },
      "ruleOptions": {
        "script/no-restricted-globals": {
          "globals": [
            {
              "name": "process",
              "message": "Read env through useRuntimeConfig()."
            },
            {
              "name": "localStorage"
            }
          ]
        }
      }
    }
  }
});
```

悪い:

```ts
const flag = process.env.FEATURE_FLAG;
const token = localStorage.getItem("auth.token");
```

良い:

```ts
const flag = useRuntimeConfig().featureFlag;
const token = authStorage.read("auth.token");
```

## `script/no-restricted-members`

SSR でも安全に使えるヘルパーへの移行など、プロジェクト内で特定のメンバーへのアクセスを禁止したいときに使います。
組み込みの禁止リストはありません。`members` が空でなく、ルールを有効にした場合だけ報告します。
各設定は、単一の識別子で表したオブジェクトと、静的なプロパティ名に一致します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  "lint": {
    "vize": {
      "rules": {
        "script/no-restricted-members": "error"
      },
      "ruleOptions": {
        "script/no-restricted-members": {
          "members": [
            {
              "object": "window",
              "property": "localStorage",
              "message": "Use authStorage."
            },
            {
              "object": "globalThis",
              "property": "process"
            }
          ]
        }
      }
    }
  }
});
```

悪い:

```ts
const token = window.localStorage.getItem("auth.token");
const env = globalThis.process.env;
```

良い:

```ts
const token = authStorage.read("auth.token");
const env = readServerEnv();
```

## `vue/component-name-in-template-casing`

テンプレートのコンポーネント名を、PascalCase または kebab-case に揃えるときに使います。
HTML/SVG の標準要素と、Vue の組み込みコンポーネントは対象外です。
以下の例は、PrimaryButton がプロジェクトで登録済みである場合の表記を比較しています。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  "lint": {
    "vize": {
      "rules": {
        "vue/component-name-in-template-casing": "warn"
      },
      "ruleOptions": {
        "vue/component-name-in-template-casing": {
          "casing": "kebab-case"
        }
      }
    }
  }
});
```

悪い（上の設定では違反）:

```vue
<template>
  <PrimaryButton />
</template>
```

良い:

```vue
<template>
  <primary-button />
</template>
```

## `script/custom-event-name-casing`

スクリプトとテンプレートの両方で、発行するカスタムイベントの名前の表記を揃えるときに使います。
既定は `camelCase` です。テンプレートで使うイベント名をハイフン区切りに統一する場合は、`kebab-case` にします。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  "lint": {
    "vize": {
      "rules": {
        "script/custom-event-name-casing": "error"
      },
      "ruleOptions": {
        "script/custom-event-name-casing": {
          "casing": "kebab-case"
        }
      }
    }
  }
});
```

悪い（上の設定では違反）:

```ts
const emit = defineEmits(["saveItem"]);
emit("saveItem");
```

良い:

```ts
const emit = defineEmits(["save-item"]);
emit("save-item");
```

## `vue/no-mutating-props`

移行中に入れ子のプロパティの変更を一時的に許可し、受け取ったプロパティ自体の再代入を禁止したい場合は、
`shallowOnly` を使います。既定の `false` では、直接の変更と入れ子のプロパティの変更を両方報告します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  "lint": {
    "vize": {
      "rules": {
        "vue/no-mutating-props": "error"
      },
      "ruleOptions": {
        "vue/no-mutating-props": {
          "shallowOnly": true
        }
      }
    }
  }
});
```

悪い（上の設定では違反）:

```vue
<script setup>
const props = defineProps<{ count: number; settings: { dense: boolean } }>();

props.count = 2;
</script>
```

良い（上の設定では許可）:

```vue
<script setup>
const props = defineProps<{ settings: { dense: boolean } }>();

props.settings.dense = true;
</script>
```

## `vue/sfc-element-order`

プロジェクトで SFC のブロックの順序を揃えるときに使います。文字列は順序の一つの位置を表し、
入れ子の配列は、その位置に配列内のどのブロックが現れてもよいことを表します。組み込みの指定は `script`、
`script:not([setup])`、`script[setup]`、`template`、`style` です。それ以外の空でない文字列は、
カスタムブロックの名前に一致します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  "lint": {
    "vize": {
      "rules": {
        "vue/sfc-element-order": "warn"
      },
      "ruleOptions": {
        "vue/sfc-element-order": {
          "order": [
            "template",
            "script:not([setup])",
            "script[setup]",
            "i18n",
            "style"
          ]
        }
      }
    }
  }
});
```

悪い（上の設定では違反）:

```vue
<style scoped></style>
<template></template>
<script setup></script>
```

良い:

```vue
<template></template>
<script setup></script>
<style scoped></style>
```

## `vue/html-self-closing`

要素の種類ごとに、自己終了タグの使い方を指定します。`always` は、内容が空の要素に自己終了タグを要求します。
`never` は開始タグと終了タグの組を要求し、`any` はどちらも許可します。
入れ子のフィールドを省略すると、そのフィールドには Vize の既定値を使います。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  "lint": {
    "vize": {
      "rules": {
        "vue/html-self-closing": "warn"
      },
      "ruleOptions": {
        "vue/html-self-closing": {
          "html": {
            "void": "always",
            "normal": "never",
            "component": "always"
          },
          "svg": "always",
          "math": "always"
        }
      }
    }
  }
});
```

悪い（上の設定では違反）:

```vue
<template>
  <div />
  <PrimaryButton></PrimaryButton>
</template>
```

良い:

```vue
<template>
  <div></div>
  <PrimaryButton />
</template>
```

## `vue/v-on-event-hyphenation`

コンポーネントの静的なカスタムイベントリスナー名に使います。`always` は `@saveItem` のような
camelCase のリスナーの引数を報告し、`never` はハイフン区切りの引数を報告します。
HTML の標準イベント、オブジェクト形式、動的な引数は対象外です。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  "lint": {
    "vize": {
      "rules": {
        "vue/v-on-event-hyphenation": "warn"
      },
      "ruleOptions": {
        "vue/v-on-event-hyphenation": "always"
      }
    }
  }
});
```

悪い（上の設定では違反）:

```vue
<template>
  <PrimaryButton @saveItem="save" />
</template>
```

良い:

```vue
<template>
  <PrimaryButton @save-item="save" />
  <button @saveItem="save" />
</template>
```

## `vue/attribute-hyphenation`

カスタムコンポーネントの静的なプロパティの属性名に使います。`always` は camelCase で書いた属性を、
`never` はハイフン区切りで書いた属性を報告します。標準の属性、`aria-*`、`data-*`、
大文字と小文字が混在する SVG の属性、動的な引数、属性として解析されたディレクティブの省略形は対象外です。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  "lint": {
    "vize": {
      "rules": {
        "vue/attribute-hyphenation": "warn"
      },
      "ruleOptions": {
        "vue/attribute-hyphenation": "never"
      }
    }
  }
});
```

悪い（上の設定では違反）:

```vue
<template>
  <UserCard user-name="Ada" />
</template>
```

良い:

```vue
<template>
  <UserCard userName="Ada" aria-label="Ada Lovelace" />
</template>
```

## `vue/valid-v-slot`

既定では、`#item.memo` や `#[name].memo` の修飾子を報告します。
Vuetify のデータテーブルなど、ドットを含むスロット名を使うコンポーネントでは、
`allowModifiers: true` を指定します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      rules: { "vue/valid-v-slot": "error" },
      ruleOptions: { "vue/valid-v-slot": { allowModifiers: true } },
    },
  },
});
```

この設定で許可される例:

```vue
<template>
  <FancyPanel>
    <template #item.memo="{ item }">{{ item.memo }}</template>
    <template #[name].memo="slot">{{ slot }}</template>
  </FancyPanel>
</template>
```

`v-slot.memo` のように引数がない場合や、`#[slot.name]="slot"` のように
スロット名がそのスロット自身のプロパティに依存する場合は、この設定でも報告します。
ファイルごとの `entries` に `allowModifiers: false` を指定すると、共通設定の
`true` より優先されます。`ruleOptions` の指定だけではルールは有効になりません。

## `musea/prefer-design-tokens`

Musea のスタイル検査で、プロジェクトのデザイントークンを使うための設定です。トークンのデータを指定し、
ルールを有効にするか、空でないトークンのリストで選択した場合だけ報告します。各トークンは、
直接書いた CSS の値を、`path` から導く CSS カスタムプロパティに対応させます。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  "lint": {
    "vize": {
      "rules": {
        "musea/prefer-design-tokens": "warn"
      },
      "ruleOptions": {
        "musea/prefer-design-tokens": {
          "tokens": [
            {
              "path": "color.primary",
              "value": "#3b82f6"
            },
            {
              "path": "color.danger",
              "value": "#ef4444",
              "tier": "semantic"
            }
          ]
        }
      }
    }
  }
});
```

悪い（上の設定では違反）:

```vue
<style scoped>
.button {
  color: #3b82f6;
}
</style>
```

良い:

```vue
<style scoped>
.button {
  color: var(--color-primary);
}
</style>
```

## `script/define-props-destructuring`

既定の `only-when-assigned` は、戻り値を変数に代入するときに分割代入を要求します。
代入しない `defineProps<Props>()` は許可します。`always` は代入しない呼び出しも検出し、
`never` は分割代入を検出します。Vue 3.5 では props の分割代入がリアクティビティを保ちます。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
export default defineConfig({
  vue: {
    linter: {
      rules: { "script/define-props-destructuring": "warn" },
      ruleOptions: { "script/define-props-destructuring": { destructure: "only-when-assigned" } },
    },
  },
});
```

悪い:

```vue
<script setup lang="ts">
const props = defineProps<{ size?: string }>();
</script>
```

良い:

```vue
<script setup lang="ts">
const { size = "md" } = defineProps<{ size?: string }>();
</script>
```

ESLint からの移行では、ルール名とオプションの指定位置を変更します。
追加する `rules` と `ruleOptions` は、上記の `vue.linter` に指定してください。

```ts annotate="remove:1;add:2,3"
 rules: { "vue/define-props-destructuring": ["warn", "always"] }
 rules: { "script/define-props-destructuring": "warn" },
 ruleOptions: { "script/define-props-destructuring": { destructure: "always" } }
```

## `vue/require-component-registration`

アプリケーションのプラグインや Musea `previewSetup` が登録するコンポーネント名を指定します。
PascalCase と kebab-case の表記を許可し、正規表現は解釈しません。オプションだけでは
ルールは有効になりません。ファイルごとの設定はリスト全体を置き換え、空のリストで引き継いだ値を消去します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "rules": {
        "vue/require-component-registration": "warn"
      },
      "ruleOptions": {
        "vue/require-component-registration": {
          "globals": [
            "MyButton",
            "MyIcon"
          ]
        }
      }
    },
  },
});
```

悪い（未登録のコンポーネント）:

```vue
<MissingWidget />
```

良い（グローバル名を指定したコンポーネント）:

```vue
<MyButton />
```
