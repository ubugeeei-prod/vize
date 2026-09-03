/**
 * Sibling suites the `test.ts` integration entry pulls in.
 *
 * The list lives here rather than inline in `test.ts` so registering another
 * suite never has to grow that entry file, which is already far over the
 * repository's source-length limit.
 */
await Promise.all([
  import("../cli/files.test.ts"),
  import("../cli/output.test.ts"),
  import("../cli/oxlint.test.ts"),
  import("../sfc-blocks.test.ts"),
  import("../vite-plus-flat-config.test.ts"),
  import("../vite-plus-lint.test.ts"),
  import("../nuxt-preset.test.ts"),
]);
