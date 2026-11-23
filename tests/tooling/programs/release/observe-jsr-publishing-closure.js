import fs from "node:fs";
import vm from "node:vm";
import { stripTypeScriptTypes } from "node:module";

const input = JSON.parse(fs.readFileSync(0, "utf8"));
const imports = [],
  calls = [],
  logs = [];
const record = (entry, args) => calls.push({ entry, args });
const context = vm.createContext({
  process: { argv: ["node", "consumer.ts", input.root, ...input.flags] },
  console: { log: (line) => logs.push(line) },
});
const exports = new Map([
  [
    input.root,
    {
      defineConfig: (...args) => {
        record("defineConfig", args);
        return input.fault === "config" ? { compiler: { vapor: true } } : args[0];
      },
    },
  ],
  [
    input.root + "/config",
    {
      loadConfig:
        input.fault === "loadConfig"
          ? null
          : () => {
              throw new Error("consumer must not load checkout configuration");
            },
    },
  ],
  [
    input.root + "/native",
    {
      compileSfc: (...args) => {
        record("compileSfc", args);
        return {
          errors: input.fault === "errors" ? ["injected"] : [],
          code: input.fault === "code" ? "" : args[0],
        };
      },
    },
  ],
  [
    input.root + "/babel",
    {
      default: input.fault === "babel" ? null : () => ({ name: "sealed-babel-facade" }),
    },
  ],
  [
    "@vizejs/babel-plugin",
    {
      default: input.fault === "babel" ? null : () => ({ name: "sealed-babel-package" }),
    },
  ],
  [
    input.root + "/oxlint",
    {
      default: input.fault === "oxlint" ? null : { meta: { name: "sealed-oxlint-facade" } },
      configs: input.fault === "oxlint-config" ? {} : { recommended: { "vize/test": "error" } },
    },
  ],
  [
    "oxlint-plugin-vize",
    {
      default: input.fault === "oxlint" ? null : { meta: { name: "sealed-oxlint-package" } },
      configs: input.fault === "oxlint-config" ? {} : { recommended: { "vize/test": "error" } },
    },
  ],
  [
    input.root + "/vite",
    {
      default: (...args) => {
        record("vite", args);
        return input.fault === "plugins" ? [] : [{ name: "sealed-facade" }];
      },
    },
  ],
]);
try {
  const module = new vm.SourceTextModule(stripTypeScriptTypes(input.source), {
    context,
    importModuleDynamically: async (specifier) => {
      imports.push(specifier);
      if (!exports.has(specifier)) throw new Error("Unexpected consumer import: " + specifier);
      const values = exports.get(specifier);
      const imported = new vm.SyntheticModule(
        Object.keys(values),
        function () {
          for (const [key, value] of Object.entries(values)) this.setExport(key, value);
        },
        { context },
      );
      await imported.link(() => {
        throw new Error("facade must not import modules");
      });
      await imported.evaluate();
      return imported;
    },
  });
  if (module.dependencySpecifiers.length) throw new Error("consumer must not have static imports");
  await module.link(() => {
    throw new Error("consumer must not have static imports");
  });
  await module.evaluate();
  process.stdout.write(JSON.stringify({ imports, calls, logs }) + "\n");
} catch (error) {
  process.stdout.write(JSON.stringify({ imports, calls, logs, error: error.message }) + "\n");
  process.exitCode = 1;
}
