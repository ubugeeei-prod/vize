const [entry, rootDir, rawOptions] = process.argv.slice(2);
let resolved = true;
try {
  import.meta.resolve("oxlint-plugin-vize");
} catch {
  resolved = false;
}
if (resolved) throw new Error("oxlint-plugin-vize must not resolve");
const { default: nuxtModule } = await import(entry);
await nuxtModule(JSON.parse(rawOptions), {
  _version: "3.19.3",
  options: { rootDir, buildDir: rootDir + "/.nuxt", srcDir: rootDir, dev: true, modules: [] },
  hook() {},
  callHook: async () => {},
});
