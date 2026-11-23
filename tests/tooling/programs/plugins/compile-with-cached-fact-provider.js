import { createRequire } from "node:module";

const fixture = JSON.parse(process.argv[2]);

const { defineFactProvider, definePlugin } = await import(fixture.sdkModule);
const native = createRequire(import.meta.url)(fixture.nativeLibrary);
const color = process.argv[4];
let calls = 0;
const provider = defineFactProvider({
  name: "tokens",
  version: "disk-provider-1",
  visit: ["ui.element"],
  provides: ["tokens/colors"],
  cacheInputs: [{ name: "palette", value: color }],
  provide(batch) {
    calls += 1;
    return { "tokens/colors": batch.nodes.map((node) => [node.id, color]) };
  },
});
const rule = definePlugin({
  name: "disk-fact-rule",
  version: "1",
  visit: ["ui.element"],
  demands: ["tokens/colors"],
  cacheInputs: [],
  rules: {
    contrast(ctx) {
      const colors = ctx.facts("tokens/colors");
      for (const node of ctx.nodes) ctx.report(node, colors.get(node.id));
    },
  },
});
const result = native.lintWithPlugins("<template><button>Go</button></template>", [rule], {
  filename: "DiskProvider.vue",
  cache: true,
  cacheDir: process.argv[3],
  factProviders: [provider],
});
process.stdout.write(
  JSON.stringify({
    calls,
    diagnostics: result.diagnostics,
    provider: result.factProviders[0],
    rule: result.plugins[0],
  }),
);
