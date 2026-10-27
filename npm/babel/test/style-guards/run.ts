import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { activeNative } from "../hosts/environment.ts";
import { hasJsx, wholeError } from "../hosts/contracts.ts";
import { hash, json } from "../runtime/environment.ts";
import { cases, source, attributes, reference } from "./cases.ts";
import { browserCases } from "./browser.ts";
import { nativeOrigins } from "./maps.ts";
import type { PluginTarget } from "@babel/core";
import type { BabelResult } from "../host-types.ts";
import type { RuntimeInput, RuntimeLoader, TemplateCompiler } from "../runtime/types.ts";
import type { NativeCall, StyleReport, StylePacket } from "./types.ts";
const factory = (module: PluginTarget | { default: PluginTarget }): PluginTarget =>
  typeof module === "function" ? module : (module as { default: PluginTarget }).default;
export async function run(input: RuntimeInput, load: RuntimeLoader) {
  const report: StyleReport = { input, packets: [], browser: null, error: null };
  const directory = path.join(input.output, input.id + "-artifacts");
  fs.mkdirSync(directory);
  try {
    const core = await load.core(),
      plugin = factory(await load.plugin()),
      typescript = factory(await load.typescript());
    assert.equal(core.version, input.compiler.version);
    const preparation = await activeNative(input);
    const native: typeof import("@vizejs/native") = load.requireHost("@vizejs/native"),
      actualCompile = native.compileJsx;
    const addons = Object.keys(load.requireHost.cache)
      .filter((file) => file.endsWith(".node"))
      .map((file) => ({ path: fs.realpathSync(file), sha256: hash(fs.readFileSync(file)) }));
    assert.deepEqual(addons, [{ path: preparation.image, sha256: preparation.preparation.sha256 }]);
    const calls: NativeCall[] = [];
    native.compileJsx = (text, options) => {
      const result = actualCompile(text, options);
      calls.push({ text, options, result });
      return result;
    };
    const fromRuntime = createRequire(path.join(input.runtime.root, "package.json"));
    const fromSfc = createRequire(fromRuntime.resolve("@vue/compiler-sfc"));
    const compiler: TemplateCompiler = fromSfc("@vue/compiler-vapor");
    try {
      for (const row of cases) {
        const authored = source(row),
          before = calls.length;
        const packet: StylePacket = {
          id: row.id,
          source: authored,
          role: row.spelling
            ? "html-case-source-refusal"
            : row.fold
              ? "admitted-literal"
              : "cssom-lexical-refusal",
          preparation,
          result: null,
          native: null,
          references: [],
        };
        report.packets.push(packet);
        packet.result = core.transformSync(authored, {
          filename: path.join(directory, row.id + ".tsx"),
          sourceFileName: row.id + ".tsx",
          sourceMaps: true,
          ast: true,
          babelrc: false,
          configFile: false,
          parserOpts: { plugins: ["jsx", "typescript"] },
          plugins: [
            [plugin, { jsxMode: "vapor" }],
            [typescript, { isTSX: true }],
          ],
        }) as BabelResult | null;
        assert(packet.result && typeof packet.result.code === "string" && packet.result.map);
        assert.equal(calls.length, before + 1);
        packet.native = calls.at(-1)!;
        assert.deepEqual(packet.native.result.errors, []);
        assert.deepEqual(packet.native.result.warnings, []);
        packet.nativeOrigins = nativeOrigins(core, packet.native, row);
        assert.equal(hasJsx(packet.result.ast), false);
        assert.deepEqual(packet.result.metadata.vize!.jsx, {
          defaultMode: "vapor",
          ssr: false,
          nativePasses: 1,
          warnings: [],
          scopedStyles: [],
        });
        assert(packet.result.map.sourcesContent!.includes(authored));
        assert.deepEqual(await activeNative(input), preparation);
        assert.deepEqual(
          Object.keys(load.requireHost.cache)
            .filter((file) => file.endsWith(".node"))
            .map((file) => fs.realpathSync(file)),
          addons.map((row) => row.path),
        );
        packet.loadedAddons = addons;
        fs.writeFileSync(path.join(directory, row.id + "-candidate.mjs"), packet.result.code);
        for (const staticStyle of row.fold ? [false, true] : [false]) {
          const template = `<button ${attributes(row, true, staticStyle)} @click="increment">{{count}}</button>`;
          const errors: string[] = [],
            warnings: string[] = [];
          const result = compiler.compile(template, {
            mode: "module",
            prefixIdentifiers: true,
            vapor: true,
            comments: true,
            onError: (error) => errors.push(String(error)),
            onWarn: (warning) => warnings.push(String(warning)),
          });
          assert.deepEqual(errors, []);
          assert.deepEqual(warnings, []);
          const kind = staticStyle ? "static" : "dynamic";
          packet.references.push({ kind, template, result, errors, warnings });
          fs.writeFileSync(path.join(directory, row.id + "-" + kind + "-render.mjs"), result.code);
          fs.writeFileSync(
            path.join(directory, row.id + "-" + kind + ".mjs"),
            reference(row, staticStyle),
          );
        }
        json(path.join(directory, row.id + "-compile.json"), packet);
      }
    } finally {
      native.compileJsx = actualCompile;
    }
    report.browser = await browserCases(input, directory, cases);
  } catch (error) {
    report.error = wholeError(error);
  } finally {
    json(input.report, report);
  }
  assert.equal(report.error, null, input.report);
}
