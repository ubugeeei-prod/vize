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
  import("../cli/locations.test.ts"),
  import("../original-locations.test.ts"),
  import("../cli/oxlint.test.ts"),
  import("../cli/scoped-config.test.ts"),
  import("../cli/scoped-discovery.test.ts"),
  import("../cli/project-checks.test.ts"),
  import("../cli/project-json.test.ts"),
  import("../cli/project-failure.test.ts"),
  import("../cli/project-failure-cli.test.ts"),
  import("../project-transport.test.ts"),
  import("../script-safe-transport.test.ts"),
  import("../discovered-transport.test.ts"),
  import("../cli/scoped-options.test.ts"),
  import("../scoped-transport.test.ts"),
  import("../sfc-blocks.test.ts"),
  import("../template-locations.test.ts"),
  import("../vite-plus-flat-config.test.ts"),
  import("../vite-plus-lint.test.ts"),
  import("../nuxt-preset.test.ts"),
]);
