export default {
  lint: {
    rules: {
      // The repository's existing public APIs use camelCase events. Keep that
      // compatibility explicit while the product default remains lowercase.
      "vize/script/custom-event-name-casing": ["error", { casing: "camelCase" }],
    },
  },
};
