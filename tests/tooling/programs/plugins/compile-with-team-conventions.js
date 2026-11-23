import { createRequire } from "node:module";

const fixture = JSON.parse(process.argv[2]);

const native = createRequire(import.meta.url)(fixture.nativeLibrary);
const { default: team } = await import(fixture.pluginModule);
const output = native.lintWithPlugins(fixture.source, [team], {
  filename: "TodoList.vue",
  cache: true,
  cacheDir: process.argv[3],
});
process.stdout.write(
  JSON.stringify({
    diagnostics: output.diagnostics,
    plugin: {
      contentKey: output.plugins[0].contentKey,
      cached: output.plugins[0].cached,
    },
  }),
);
