---
title: ルール オプション
---
<!-- Generated translation; source: rules/options.md -->

# ルール オプション
<!-- Type-aware reference: [型ルール オプション](./options-type.md) -->

Vite+ では `vite.config.ts` の `lint.vize.ruleOptions` に指定し、`vp run lint` で検査します。standalone の互換設定では同じ内容を `linter.ruleOptions` に指定します。

`linter.ruleOptions` は、オプションを受け取るルールの project-local な型付き設定です。
未知の option field は拒否されます。同じ rule に対して後から一致した config entry がある場合、その rule の option object 全体を置き換えます。重大度は `linter.rules` で設定し、
`ruleOptions` は有効なルールの振る舞いだけを決めます。

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

| ルール | Option shape | 既定値と挙動 |
| --- | --- | --- |
| `html/no-empty-palpable-content` | `{ contentDirectives?: string[] }` | 既定は空 list です。可視 content を供給する directive の bare name を完全一致で指定します。rule は別途有効にしてください。 |
| `script/no-restricted-globals` | `{ globals?: Array<{ name: string; message?: string }> }` | option なしでは組み込み deny list の `process`、`localStorage`、`sessionStorage` を使います。空でない `globals` は組み込み list を置き換えます。 |
| `script/no-restricted-members` | `{ members?: Array<{ object: string; property: string; message?: string }> }` | `members` が設定され、かつ rule が有効なときだけ発火します。`message` がない場合は汎用の help を使います。 |
| `vue/component-name-in-template-casing` | `{ casing?: "PascalCase" \| "kebab-case" }` | 既定は `PascalCase` です。 |
| `script/custom-event-name-casing` | `{ casing?: "camelCase" \| "kebab-case" }` | 既定は `camelCase` です。 |
| `script/define-props-destructuring` | `{ mode?: "only-when-assigned" \| "always" \| "never" }` | 既定は `only-when-assigned` です。`always` は代入しない呼び出しも検査し、`never` は props オブジェクトでの参照を要求します。Vue 3.5 の分割代入は反応性を保ちます。 |
| `vue/no-mutating-props` | `{ shallowOnly?: boolean }` | 既定は `false` です。`true` では direct prop replacement を禁止したまま nested mutation は許可します。 |
| `vue/require-component-registration` | `{ globals?: string[] }` | application plugin や previewSetup が登録する component 名を指定します。PascalCase と kebab-case を許可します。rule は別途有効にしてください。 |
| `vue/sfc-element-order` | `{ order?: Array<string \| string[]> }` | 既定は `[["script", "template"], "style"]` です。ネストした配列は、その rank でどの selector でもよいことを表します。 |
| `vue/html-self-closing` | `{ html?: { void?: Style; normal?: Style; component?: Style }; svg?: Style; math?: Style }`, where `Style` is `"always"`, `"never"`, or `"any"` | 既定は `html.void: "always"`、`html.normal: "any"`、`html.component: "always"`、`svg: "always"`、`math: "always"` です。 |
| `vue/v-on-event-hyphenation` | `"always" \| "never"` | component 上の static event listener 名を hyphenation するかを設定します。 |
| `vue/attribute-hyphenation` | `"always" \| "never"` | template 内の component prop attribute を hyphenation するかを設定します。 |
| `musea/prefer-design-tokens` | `{ tokens?: Array<{ path: string; value: string; tier?: string }> }` | token data が設定され、rule が有効か、空でない token list で暗黙に選択されたときだけ発火します。`tier` の既定は `primitive` です。 |
| `type/strict-boolean-expressions` | `{ allowString?: boolean; allowNumber?: boolean; allowNullableObject?: boolean; allowNullableBoolean?: boolean; allowNullableString?: boolean; allowNullableNumber?: boolean; allowNullableEnum?: boolean; allowAny?: boolean }` | `allowString`、`allowNumber`、`allowNullableObject` の既定は `true`、その他は `false` です。rule を明示的に有効にする必要があります。 |

## `html/no-empty-palpable-content`

custom directive は focus、style、content の供給などを行えます。可視 content を供給すると
project が保証できる directive だけを指定してください。`v-` を除いた template の名前に
完全一致します。argument と modifier はその名前を変えません。空 list は既定動作に戻し、
scoped entry は option object 全体を置き換えます。sanitization の保証や rule の有効化は行いません。

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

悪い（未登録の content source）:

```vue
<p v-sanitize="message" />
```

良い（設定した provider）:

```vue
<p v-safe-html="message" />
```

## Scoped Entries

`entries` が file に一致した場合、その `linter.ruleOptions` が root option に overlay されます。
同じ rule の option object は丸ごと置き換えられます。`globals`、`members`、`order`、`tokens`
などの配列は entry 間で連結されません。

## `script/no-restricted-globals`

runtime global を project-owned wrapper 経由にしたいときに使います。option がない場合は
`process`、`localStorage`、`sessionStorage` を報告します。空でない `globals` はその組み込み
list を置き換えます。空 list は組み込み list に戻るため、設定ミスで rule が黙って無効にはなりません。

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

SSR-safe helper への移行など、project-local な member access ban に使います。組み込み deny list は
なく、`members` が空でなく、かつ rule が有効なときだけ報告します。各 entry は bare identifier
の receiver と static property name に一致します。

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

template 内の component tag を PascalCase または kebab-case に揃えるときに使います。native
HTML/SVG 要素と Vue built-in は対象外です。

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

上の config では悪い:

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

script と template usage の両方で、emitted custom event を同じ casing に揃えるときに使います。
既定は `camelCase` です。template-facing な event 名で統一したい project では `kebab-case` にします。

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

上の config では悪い:

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

移行中に nested prop mutation は一時的に許可しつつ、prop binding 自体の置き換えは禁止したい場合は
`shallowOnly` を使います。既定の `false` では direct mutation と nested mutation の両方を報告します。

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

上の config では悪い:

```vue
<script setup>
const props = defineProps<{ count: number; settings: { dense: boolean } }>();

props.count = 2;
</script>
```

上の config では良い:

```vue
<script setup>
const props = defineProps<{ settings: { dense: boolean } }>();

props.settings.dense = true;
</script>
```

## `vue/sfc-element-order`

project 固有の SFC block order がある場合に使います。文字列は 1 つの rank、ネストした配列は
その rank でどの block selector でもよいことを表します。組み込み selector は `script`、
`script:not([setup])`、`script[setup]`、`template`、`style` です。他の空でない文字列は
custom block 名に一致します。

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

上の config では悪い:

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

element family ごとの self-closing style を選ぶための option です。`always` は空要素に self-closing
を要求し、`never` は start/end tag の pair を要求し、`any` はどちらも受け入れます。省略した nested
field は Vize の既定値を維持します。

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

上の config では悪い:

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

component 上の static custom event listener 名に使います。`always` は `@saveItem` のような
camelCase listener argument を報告し、`never` は hyphenated listener argument を報告します。
native HTML event、object syntax、dynamic argument は対象外です。

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

上の config では悪い:

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

custom component 上の static prop name に使います。`always` は camelCase の authored attribute、
`never` は hyphenated authored attribute を報告します。native attribute、`aria-*`、`data-*`、
SVG の mixed-case attribute、dynamic argument、attribute として parse された directive shorthand は対象外です。

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

上の config では悪い:

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

## `musea/prefer-design-tokens`

Musea/style lint を project の design-token inventory に接続するための option です。token data が
設定され、rule が有効か、空でない token list で選択されたときだけ発火します。各 token は
hardcoded CSS value を `path` から導かれる CSS custom property に対応させます。

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

上の config では悪い:

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
`never` は分割代入を検出します。Vue 3.5 では props の分割代入が反応性を保ちます。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";
export default defineConfig({
  lint: {
    vize: {
      rules: { "script/define-props-destructuring": "warn" },
      ruleOptions: { "script/define-props-destructuring": { mode: "only-when-assigned" } },
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
追加する `rules` と `ruleOptions` は、上記の `lint.vize` に指定してください。

```diff
- rules: { "vue/define-props-destructuring": ["warn", "always"] }
+ rules: { "script/define-props-destructuring": "warn" },
+ ruleOptions: { "script/define-props-destructuring": { mode: "always" } }
```

## `vue/require-component-registration`

application plugin や Musea `previewSetup` が登録する component 名を明示します。
PascalCase と kebab-case を許可し、正規表現は解釈しません。option だけでは
rule は有効になりません。scoped entry は list 全体を置き換え、空 list で reset します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
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

悪い（未登録の component）:

```vue
<MissingWidget />
```

良い（設定した global）:

```vue
<MyButton />
```
