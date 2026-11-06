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
    },
  },
};
