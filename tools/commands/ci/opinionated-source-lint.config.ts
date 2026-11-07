export default {
  lint: {
    rules: {
      // The repository's existing public APIs use camelCase events. Keep that
      // compatibility explicit while the product default remains lowercase.
      "vize/script/custom-event-name-casing": ["error", { casing: "camelCase" }],
      // Existing repository apps still support props-object and pre-3.5
      // patterns. Keep that migration local while the product default requires
      // reactive props destructuring.
      "vize/script/define-props-destructuring": "off",
      // The product preset intentionally stays strict. The monorepo's existing
      // authored apps have not completed these migrations yet, so keep the
      // repository CI exception explicit instead of weakening the preset.
      "vize/script/no-boolean-default": "off",
      "vize/script/no-unused-emit-declarations": "off",
      "vize/script/require-default-prop": "off",
      "vize/script/require-explicit-slots": "off",
      "vize/script/require-typed-object-prop": "off",
      "vize/vue/html-self-closing": "off",
      "vize/vue/multi-word-component-names": "off",
      "vize/vue/no-negated-v-if-condition": "off",
      "vize/vue/no-root-v-if": "off",
      "vize/vue/prefer-true-attribute-shorthand": "off",
      "vize/vue/require-component-is": "off",
      "vize/vue/v-on-handler-style": "off",
    },
  },
};
