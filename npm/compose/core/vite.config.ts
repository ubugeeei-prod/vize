import { defineConfig } from "vite-plus";

export default defineConfig({
  lint: {
    ignorePatterns: ["dist/**"],
    options: { typeAware: true },
  },
  fmt: { ignorePatterns: ["dist/**"] },
  pack: {
    entry: {
      index: "src/index.ts",
      "async-resource": "src/async-resource.ts",
      capability: "src/capability.ts",
      "disposal-scope": "src/disposal-scope.ts",
      "event-listener": "src/event-listener.ts",
      locale: "src/locale.ts",
      "media-query": "src/media-query.ts",
      scope: "src/scope.ts",
      temporal: "src/temporal.ts",
      "timeout-scheduler": "src/timeout-scheduler.ts",
      "use-counter": "src/use-counter.ts",
      "use-debounced": "src/use-debounced.ts",
      "use-history": "src/use-history.ts",
      "use-previous": "src/use-previous.ts",
      "use-throttled": "src/use-throttled.ts",
      "use-toggle": "src/use-toggle.ts",
    },
    format: "esm",
    dts: true,
    clean: true,
    deps: {
      neverBundle: ["vue"],
    },
  },
});
