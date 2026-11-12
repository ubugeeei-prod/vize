export default {
  lint: {
    rules: {
      // Existing app examples use <output> as a live status value without a
      // separately named form control. Keep that repository migration local
      // while the product preset requires labels for output, meter and progress.
      "vize/a11y/form-control-has-label": "off",
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
      // #12878 added this rule to the opinionated preset. It does not yet see
      // components used inside Musea <art> blocks, and the remaining reports
      // (unused type imports and locals) are an app migration. Keep the
      // repository exception local until both are done.
      "vize/vue/no-unused-script-vars": "off",
      "vize/vue/prefer-true-attribute-shorthand": "off",
      "vize/vue/require-component-is": "off",
      "vize/vue/v-on-handler-style": "off",
      // Complete static-class detection now includes movable items inside
      // dynamic arrays and template literals. Existing authored apps have not
      // completed that migration, so keep the repository exception local.
      "vize/vapor/prefer-static-class": "off",
    },
  },
};
