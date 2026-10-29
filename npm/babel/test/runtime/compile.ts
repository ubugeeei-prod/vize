import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { activeNative } from "../hosts/environment.ts";
import { hasJsx, wholeError } from "../hosts/contracts.ts";
import { source } from "./fixtures.ts";
import { hash, json } from "./environment.ts";
import type { PluginTarget } from "@babel/core";
import type { BabelResult } from "../host-types.ts";
import type { RuntimeInput, RuntimeLoader, CompilePacket } from "./types.ts";

const factory = (module: PluginTarget | { default: PluginTarget }): PluginTarget =>
  typeof module === "function" ? module : (module as { default: PluginTarget }).default;
export async function compilePair(input: RuntimeInput, load: RuntimeLoader, directory: string) {
  const core = await load.core();
  const plugin = factory(await load.plugin());
  const typescript = factory(await load.typescript());
  assert.equal(core.version, input.compiler.version);
  const preparation = await activeNative(input);
  const packets: CompilePacket[] = [];
  for (const phase of ["client", "server"] as const) {
    const packet: CompilePacket = {
      source,
      phase,
      target: input.runtime.target,
      coreVersion: core.version,
      preparation,
      result: null,
      error: null,
      loadedAddons: [],
    };
    try {
      assert.deepEqual(await activeNative(input), preparation);
      packet.result = core.transformSync(source, {
        filename: path.join(directory, "App.tsx"),
        sourceFileName: "App.tsx",
        sourceMaps: true,
        ast: true,
        babelrc: false,
        configFile: false,
        comments: true,
        parserOpts: { plugins: ["jsx", "typescript"] },
        plugins: [
          [plugin, { jsxMode: input.runtime.target, jsxCompat: "native", ssr: phase === "server" }],
          [typescript, { isTSX: true }],
        ],
      }) as BabelResult | null;
      assert(packet.result && typeof packet.result.code === "string");
      fs.writeFileSync(path.join(directory, phase + ".mjs"), packet.result.code);
      packet.codeSha256 = hash(Buffer.from(packet.result.code));
      packet.loadedAddons = Object.keys(load.requireHost.cache)
        .filter((file) => file.endsWith(".node"))
        .map((file) => ({ path: fs.realpathSync(file), sha256: hash(fs.readFileSync(file)) }));
      assert.deepEqual(packet.loadedAddons, [
        { path: preparation.image, sha256: preparation.preparation.sha256 },
      ]);
      assert.deepEqual(await activeNative(input), preparation);
      assert.equal(hasJsx(packet.result.ast), false);
      assert.deepEqual(packet.result.metadata.vize!.jsx, {
        defaultMode: input.runtime.target,
        ssr: phase === "server",
        nativePasses: 1,
        warnings: [],
        scopedStyles: [],
      });
      assert(
        packet.result.map && packet.result.map.sourcesContent!.includes(source),
        "actual emitted map retains authored CRLF/Unicode source",
      );
    } catch (error) {
      packet.error = wholeError(error);
    }
    packets.push(packet);
    json(path.join(directory, "compile-" + phase + ".json"), packet);
  }
  assert(
    packets.every((packet) => packet.error === null),
    "both source-native compiler phases must qualify; inspect whole packets",
  );
  return packets;
}
