## Summary

#7204 fixed `vue/prefer-props-shorthand` and `a11y/no-redundant-roles`, which were marked **Fixable: Yes** in `docs/content/rules/all.md` but produced no edits. Four more template rules still have the same problem on 0.432.0: they are listed as fixable, `RuleMeta` declares `fixable: true`, and `vize lint --fix` leaves the file unchanged. The JSON output has no fix data for them.

| Rule | Source (main @ d1a25ec) |
| --- | --- |
| `vue/html-self-closing` | `crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs` |
| `vue/component-name-in-template-casing` | `crates/vize_patina/src/rules/opinionated/vue/component_name_in_template_casing.rs` |
| `vue/v-slot-style` | `crates/vize_patina/src/rules/vue/v_slot_style.rs` |
| `vue/no-boolean-attr-value` | `crates/vize_patina/src/rules/opinionated/vue/no_boolean_attr_value.rs` |

None of these files builds a `Fix` / `TextEdit`. `vapor/require-vapor-attribute` (`crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs`) is declared and documented the same way and has no edit either. I didn't get it to report in this repro, so it isn't in the table above.

The first three are autofixable in eslint-plugin-vue (`meta.fixable: "code"`), and all four are mechanical rewrites (`<X></X>` → `<X />`, `<my-card>` → `<MyCard>`, `v-slot:header` → `#header`, `disabled="disabled"` → `disabled`).

## Environment

- `vize` 0.432.0 (npm)
- macOS 26.4.1 arm64, Node v26.10.0, npm 11.19.1

## Reproduction

```sh
mkdir -p repro-fixable && cd repro-fixable
printf '<template>\n  <div><slot name="header" /></div>\n</template>\n' > MyCard.vue
cat > CardList.vue <<'VUE'
<script setup lang="ts">
import MyCard from "./MyCard.vue";
</script>

<template>
  <MyCard></MyCard>
  <my-card />
  <MyCard>
    <template v-slot:header>Title</template>
  </MyCard>
  <input disabled="disabled" />
</template>
VUE
cat > vize.config.json <<'JSON'
{
  "linter": {
    "preset": "incremental",
    "rules": {
      "vue/html-self-closing": "warn",
      "vue/component-name-in-template-casing": "warn",
      "vue/v-slot-style": "warn",
      "vue/no-boolean-attr-value": "warn"
    }
  }
}
JSON
cp CardList.vue CardList.before.vue
npx vize@0.432.0 lint --fix -f plain --help-level none CardList.vue
diff CardList.before.vue CardList.vue && echo "CardList.vue unchanged"
```

## Actual

```text
CardList.vue:6:3 warning vue/html-self-closing Empty component should be self-closing
CardList.vue:7:3 warning vue/component-name-in-template-casing Component should use PascalCase
CardList.vue:9:15 warning vue/v-slot-style Expected '#header' instead of 'v-slot:header'
CardList.vue:11:10 warning vue/no-boolean-attr-value Boolean attribute "disabled" should not have value "disabled"
CardList.vue unchanged
```

## Expected

Either `--fix` rewrites the template to

```vue
<template>
  <MyCard />
  <MyCard />
  <MyCard>
    <template #header>Title</template>
  </MyCard>
  <input disabled />
</template>
```

or the rules are marked `fixable: false` and the rules table stops listing them as fixable.
