---
title: Rule Options
---

# Rule Options

`linter.ruleOptions` holds typed project-local settings for rules that accept options. Unknown
option fields are rejected, and later matching config entries replace the full option object for the
same rule. Set severity separately under `linter.rules`; options only describe how an enabled rule
should behave unless noted below.

```json
{
  "linter": {
    "rules": {
      "script/no-restricted-globals": "error",
      "vue/html-self-closing": "warn",
      "musea/prefer-design-tokens": "warn"
    },
    "ruleOptions": {
      "script/no-restricted-globals": {
        "globals": [
          { "name": "process", "message": "Read env via a typed helper." },
          { "name": "alert" }
        ]
      },
      "vue/html-self-closing": {
        "html": { "void": "always", "normal": "never", "component": "always" },
        "svg": "always",
        "math": "always"
      },
      "musea/prefer-design-tokens": {
        "tokens": [
          { "path": "color.primary", "value": "#3b82f6", "tier": "semantic" }
        ]
      }
    }
  }
}
```

| Rule | Option shape | Defaults and behavior |
| --- | --- | --- |
| `script/no-restricted-globals` | `{ globals?: Array<{ name: string; message?: string }> }` | Without options, the built-in deny list is `process`, `localStorage`, and `sessionStorage`. A non-empty `globals` list replaces that built-in list. |
| `script/no-restricted-members` | `{ members?: Array<{ object: string; property: string; message?: string }> }` | Off unless `members` is configured and the rule is enabled. A missing `message` uses the generic diagnostic help. |
| `vue/component-name-in-template-casing` | `{ casing?: "PascalCase" \| "kebab-case" }` | Defaults to `PascalCase`. |
| `script/custom-event-name-casing` | `{ casing?: "camelCase" \| "kebab-case" }` | Defaults to `camelCase`. |
| `vue/no-mutating-props` | `{ shallowOnly?: boolean }` | Defaults to `false`; `true` allows nested mutation while still disallowing direct prop reassignment. |
| `vue/sfc-element-order` | `{ order?: Array<string \| string[]> }` | Defaults to `[["script", "template"], "style"]`. A nested array means any of those block selectors may appear at that rank. |
| `vue/html-self-closing` | `{ html?: { void?: Style; normal?: Style; component?: Style }; svg?: Style; math?: Style }`, where `Style` is `"always"`, `"never"`, or `"any"` | Defaults are `html.void: "always"`, `html.normal: "any"`, `html.component: "always"`, `svg: "always"`, and `math: "always"`. |
| `vue/v-on-event-hyphenation` | `"always" \| "never"` | Configures whether event listener names should be hyphenated. |
| `vue/attribute-hyphenation` | `"always" \| "never"` | Configures whether component prop attributes should be hyphenated in templates. |
| `musea/prefer-design-tokens` | `{ tokens?: Array<{ path: string; value: string; tier?: string }> }` | Off unless token data is configured and the rule is enabled or implicitly selected by a non-empty token list. `tier` defaults to `primitive`. |

## Detailed Examples

- [Script Rule Options](./options-script.md): restricted globals, restricted members, and custom
  event casing.
- [Vue Rule Options](./options-vue.md): component casing, prop mutation depth, SFC block order,
  self-closing tags, event names, and attribute casing.
- [Musea Rule Options](./options-musea.md): design-token inventory configuration.


## Scoped Entries

When `entries` match a file, their `linter.ruleOptions` overlay the root options for that file.
Each configured rule option object replaces the same root object; arrays such as `globals`,
`members`, `order`, and `tokens` are not concatenated across entries.
