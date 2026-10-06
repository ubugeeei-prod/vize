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
