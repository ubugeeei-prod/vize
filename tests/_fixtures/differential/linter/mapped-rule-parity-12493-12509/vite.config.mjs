export default {
  vue: {
    linter: {
      preset: "incremental",
      rules: {
        "script/custom-event-name-casing": "error",
        "script/no-async-in-computed": "error",
        "script/no-dupe-keys": "error",
        "script/no-side-effects-in-computed-properties": "error",
        "vue/no-dupe-v-else-if": "error",
        "vue/no-mutating-props": "error",
        "vue/require-v-for-key": "error",
        "vue/valid-v-for": "error",
        "vue/valid-v-memo": "error"
      }
    }
  }
}
