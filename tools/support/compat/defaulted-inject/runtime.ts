import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const [provider, output] = process.argv.slice(2);
assert(provider && output, "fresh pinned public provider and output are required");
assert.equal(process.env.NODE_ENV, "development");
const require = createRequire(path.join(path.resolve(provider), "package.json"));
const vue = require("vue") as {
  version: string;
  inject: (...args: unknown[]) => unknown;
  createSSRApp: (options: { render: () => null }) => {
    runWithContext: (fn: () => unknown) => unknown;
  };
};
assert.equal(vue.version, "3.5.44");
const app = vue.createSSRApp({ render: () => null });
const empty: [] = [];
const flags: [] = [];
const cases = [
  { name: "required", run: () => vue.inject("state"), warns: true, value: undefined },
  {
    name: "explicit-factory-default",
    run: () => vue.inject("state", () => false, true),
    warns: false,
    value: false,
  },
  {
    name: "empty-tuple-spread",
    run: () => vue.inject("state", ...empty),
    warns: true,
    value: undefined,
  },
  {
    name: "unknown-spread-empty-runtime",
    run: () => vue.inject("state", ...([] as unknown[])),
    warns: true,
    value: undefined,
  },
  {
    name: "unknown-spread-one-runtime",
    run: () => vue.inject("state", ...([false] as unknown[])),
    warns: false,
    value: false,
  },
  {
    name: "explicit-undefined-before-flags",
    run: () => vue.inject("state", undefined, ...flags),
    warns: false,
    value: undefined,
  },
  { name: "explicit-null", run: () => vue.inject("state", null), warns: false, value: null },
];
const packets: unknown[] = [];
const previousWarn = console.warn;
try {
  for (const input of cases) {
    const warnings: string[][] = [];
    console.warn = (...args: unknown[]) => {
      warnings.push(args.map(String));
    };
    const actual = app.runWithContext(input.run);
    assert.equal(actual, input.value, input.name);
    assert.deepEqual(
      warnings,
      input.warns ? [['[Vue warn]: injection "state" not found.']] : [],
      input.name,
    );
    packets.push({
      name: input.name,
      source: input.run.toString(),
      value: actual === undefined ? { type: "undefined" } : { type: typeof actual, value: actual },
      warnings,
    });
  }
} finally {
  console.warn = previousWarn;
}
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const modules = Object.keys(require.cache)
  .filter((filename) => filename.indexOf("/@vue/") >= 0 || filename.indexOf("/vue/") >= 0)
  .sort()
  .map((filename) => ({
    path: fs.realpathSync(filename),
    sha256: hash(fs.readFileSync(filename)),
  }));
assert(modules.length > 0, "physical loaded Vue modules are required");
const vuePackage = require.resolve("vue/package.json");
fs.writeFileSync(
  output,
  JSON.stringify(
    {
      schema: "vize.optional-inject.pinned-vue-arity.v1",
      vue: {
        version: vue.version,
        package: fs.realpathSync(vuePackage),
        packageSha256: hash(fs.readFileSync(vuePackage)),
        modules,
      },
      packets,
    },
    null,
    2,
  ) + "\n",
);
console.log(JSON.stringify({ qualified: true, vue: vue.version, arityCases: packets.length }));
