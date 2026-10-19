---
title: スクリプトのルール設定
---
<!-- Reviewed translation; source: rules/options-script.md -->

<span id="script-rule-options"></span>

# スクリプトのルール設定

実行環境のグローバル変数や特定のプロパティへのアクセスを制限したいとき、イベント名を揃えたいとき、
props の分割代入の方針を決めたいときに、このページの例を使ってください。
設定できるオプションの一覧、未定義のフィールドを指定した場合の動作、ファイルごとの設定の置き換え方は、
[ルールのオプション](./options.md)で確認できます。

## `script/no-restricted-globals`

実行環境のグローバル変数を、プロジェクトで用意したヘルパー経由で使う場合に、このルールを使います。
オプションを省略すると、`process`、`localStorage`、`sessionStorage` を報告します。
空でない `globals` のリストを指定すると、組み込みのリストを置き換えます。空のリストでは組み込みの
設定に戻るため、リストを空にしてもルールの検査は無効になりません。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
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

ブラウザーの API を SSR を考慮したヘルパー経由で呼び出すなど、特定のプロパティへの直接アクセスを
プロジェクトごとに禁止する場合に、このルールを使います。組み込みの禁止リストはありません。
`members` が空でなく、ルールを有効にしている場合だけ報告します。各項目は、変数名で指定した
オブジェクトと、静的なプロパティ名の組み合わせに一致します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
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

## `script/custom-event-name-casing`

スクリプトとテンプレートで使うカスタムイベント名の表記を揃えるための設定です。
既定は `camelCase` です。テンプレートで使うハイフン区切りの表記に揃える場合は、`kebab-case` を指定します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
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

## `script/define-props-destructuring`

既定の `only-when-assigned` では、戻り値を変数に代入する場合に分割代入を要求します。
戻り値を代入しない `defineProps<Props>()` は許可します。`always` は戻り値を代入しない呼び出しも報告し、
`never` は反対に分割代入を報告します。Vue 3.5 では、props を分割代入してもリアクティビティを保ちます。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      preset: "incremental",
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

ESLint から移行する場合は、Vize のルール ID と型付きオプションを別に指定します。

```ts annotate="remove:1;add:2,3"
 rules: { "vue/define-props-destructuring": ["warn", "always"] }
 rules: { "script/define-props-destructuring": "warn" },
 ruleOptions: { "script/define-props-destructuring": { destructure: "always" } }
```

追加する設定は、上記の Vite+ 設定の `vue.linter` に指定してください。
