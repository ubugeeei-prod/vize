import { createRequire } from "node:module";

const fixture = JSON.parse(process.argv[2]);

const native = createRequire(import.meta.url)(fixture.nativeLibrary);
const { definePlugin } = await import(fixture.sdkModule);
const threshold = process.argv[4];
const plugin = definePlugin({
  name: "captured-config",
  version: "1",
  visit: ["ui.element"],
  demands: [],
  cacheInputs: [{ name: "threshold", value: threshold }],
  rules: {
    check(ctx) {
      ctx.report(ctx.nodes[0], threshold);
    },
  },
});
const output = native.lintWithPlugins(fixture.source, [plugin], {
  filename: "TodoList.vue",
  cache: true,
  cacheDir: process.argv[3],
});
process.stdout.write(
  JSON.stringify({
    fingerprint: plugin.fingerprint,
    key: output.plugins[0].contentKey,
    cached: output.plugins[0].cached,
    messages: output.diagnostics.map((diagnostic) => diagnostic.message),
  }),
);
