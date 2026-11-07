import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { activeNative, hash, json } from "./hosts/environment.ts";
import type { InputOptions, PluginItem, PluginTarget } from "@babel/core";
import type { BabelCore, BabelResult } from "./host-types.ts";
interface Sample {
  iterations: number;
  totalNs: string;
  realNativeCalls: number;
  realNativeNs: string;
  hostAndAdapterNs: string;
}

// This observer invokes the real compiler. The wrapper measures its call
// boundary; it never substitutes a renderer or changes its input/output.
const source = process.env.SOURCE_SHA;
assert.match(source ?? "", /^[a-f0-9]{40}$/);
const nativeRoot = fs.realpathSync(process.env.VIZE_BABEL_NATIVE_ROOT!);
const host = new URL("./hosts/fixtures/babel8/package.json", import.meta.url);
const requireHost = createRequire(host);
const core: BabelCore = await import(pathToFileURL(requireHost.resolve("@babel/core")).href);
const typescript: PluginTarget = (
  await import(pathToFileURL(requireHost.resolve("@babel/plugin-transform-typescript")).href)
).default;
assert.equal(core.version, "8.0.9");
const preparation = await activeNative({ nativeRoot, source: source! });
const native: typeof import("@vizejs/native") = requireHost("@vizejs/native");
const descriptor = Object.getOwnPropertyDescriptor(native, "compileJsx");
assert(descriptor && typeof descriptor.value === "function");
assert.equal(descriptor.writable, true);
const compiler = descriptor.value as typeof native.compileJsx;
let calls = 0;
let nativeNs = 0n;
native.compileJsx = function (this: typeof native, ...args: Parameters<typeof compiler>) {
  calls++;
  const start = process.hrtime.bigint();
  try {
    return Reflect.apply(compiler, this, args);
  } finally {
    nativeNs += process.hrtime.bigint() - start;
  }
};
const plugin: PluginTarget = (
  await import(pathToFileURL(requireHost.resolve("@vizejs/babel-plugin")).href)
).default;
const fixtures = [
  { name: "plain-module", jsx: false, source: "export const value = 42;\n", filename: "plain.js" },
  {
    name: "small-jsx",
    jsx: true,
    source: 'export const App = () => <p title="測定">hello</p>;\n',
    filename: "small.jsx",
  },
  {
    name: "typed-jsx",
    jsx: true,
    source: "export const App = (props: { label: string }) => <p>{props.label}</p>;\n",
    filename: "typed.tsx",
    typed: true,
  },
  {
    name: "many-authored-components",
    jsx: true,
    source: Array.from(
      { length: 40 },
      (_, i) => `export const Component${i} = () => <div data-index="${i}">item ${i}</div>;`,
    ).join("\n"),
    filename: "many.jsx",
  },
];
const rows = [];
try {
  for (const fixture of fixtures)
    for (const sourceMaps of [false, true]) {
      const baseOptions: InputOptions & { plugins: PluginItem[] } = {
        filename: fixture.filename,
        babelrc: false,
        configFile: false,
        sourceMaps,
        plugins: fixture.typed ? [[typescript, { isTSX: true, allExtensions: true }]] : [],
        parserOpts: { plugins: fixture.typed ? ["typescript", "jsx"] : ["jsx"] },
      };
      const options = { ...baseOptions, plugins: [plugin, ...baseOptions.plugins] };
      const output = core.transformSync(fixture.source, options) as BabelResult | null;
      assert(output && typeof output.code === "string");
      assert.equal(output.metadata.vize?.jsx!.nativePasses ?? 0, fixture.jsx ? 1 : 0);
      const baseline = fixture.jsx ? null : core.transformSync(fixture.source, baseOptions);
      if (baseline) {
        const { options: _a, ...actual } = output;
        const { options: _b, ...expected } = baseline;
        assert.deepEqual(actual, expected);
      }
      for (let warm = 0; warm < 5; warm++) core.transformSync(fixture.source, options);
      const samples: Sample[] = [];
      for (let sample = 0; sample < 7; sample++) {
        calls = 0;
        nativeNs = 0n;
        const observed = [];
        const start = process.hrtime.bigint();
        for (let iteration = 0; iteration < 20; iteration++) {
          observed.push(core.transformSync(fixture.source, options));
        }
        const total = process.hrtime.bigint() - start;
        for (const current of observed) {
          assert(current);
          assert.equal(current.code, output.code, "every measured transform is deterministic");
          assert.deepEqual(current.map, output.map);
        }
        assert.equal(calls, fixture.jsx ? 20 : 0, "exactly one real native call per JSX file");
        samples.push({
          iterations: 20,
          totalNs: String(total),
          realNativeCalls: calls,
          realNativeNs: String(nativeNs),
          hostAndAdapterNs: String(total - nativeNs),
        });
      }
      const median = (key: "totalNs" | "realNativeNs" | "hostAndAdapterNs") =>
        samples
          .map((sample) => BigInt(sample[key]))
          .sort((a, b) => (a < b ? -1 : a > b ? 1 : 0))[3]
          .toString();
      rows.push({
        fixture,
        sourceMaps,
        code: output.code,
        map: output.map,
        metadata: output.metadata,
        samples,
        medianTotalNs: median("totalNs"),
        medianRealNativeNs: median("realNativeNs"),
        medianHostAndAdapterNs: median("hostAndAdapterNs"),
      });
    }
  const image = Object.keys(requireHost.cache).filter((file) => file.endsWith(".node"));
  assert.equal(image.length, 1);
  assert.equal(hash(fs.readFileSync(image[0])), preparation.preparation.sha256);
  json(process.env.VIZE_BABEL_PROFILE_OUTPUT!, {
    schema: "vize.babel.source-host-profile.v1",
    scope:
      "instrumented debug source addon and complete Babel transform; no production or equivalent-work speedup claim",
    nativeBuild: "build:native:test uses native build:debug",
    source,
    host: core.version,
    node: process.version,
    preparation,
    image: { path: image[0], sha256: hash(fs.readFileSync(image[0])) },
    packedTarballSha256: hash(fs.readFileSync(process.env.VIZE_BABEL_PACKED_PLUGIN!)),
    rows,
  });
} finally {
  Object.defineProperty(native, "compileJsx", descriptor);
}
