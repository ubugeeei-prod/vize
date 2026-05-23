import { defineConfig } from "vite-plus";

export default defineConfig({
  lint: {
    ignorePatterns: ["dist/**"],
    options: {
      typeAware: true,
    },
  },
  fmt: {
    ignorePatterns: ["dist/**"],
  },
  pack: {
    entry: ["src/extension.ts"],
    outDir: "dist",
    format: "cjs",
    platform: "node",
    minify: true,
    deps: {
      neverBundle: ["vscode"],
      alwaysBundle: [/^vscode-languageclient(?:\/|$)/],
      onlyBundle: [
        "balanced-match",
        "brace-expansion",
        "minimatch",
        "semver",
        "vscode-jsonrpc",
        "vscode-languageclient",
        "vscode-languageserver-protocol",
        "vscode-languageserver-types",
      ],
    },
  },
});
