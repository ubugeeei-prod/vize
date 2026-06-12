export default {
  compiler: {
    jsxMode: "vdom",
  },
  typeChecker: {
    enabled: true,
    jsxTypecheck: true,
    strict: true,
  },
  linter: {
    enabled: true,
    categories: {
      accessibility: "error",
      correctness: "error",
      suspicious: "warn",
    },
  },
  formatter: {
    printWidth: 90,
    tabWidth: 2,
    semi: true,
    singleQuote: false,
    trailingComma: "all",
  },
};
