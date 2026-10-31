/** Three complete hosted consumer outputs; the historical snapshots stay unchanged. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { workspaceRoot } from "./helpers.ts";

type NativeDto = {
  code: string;
  errors: string[];
  warnings: string[];
  scopedStyles: { scopeId: string; css: string }[];
  map?: string;
};
type BundlePacket = {
  source: string;
  entry: string;
  nativeDto: NativeDto;
  mapOwnProperty: boolean;
  mapIsUndefined: boolean;
  loadedModule: string;
  bundle: string;
  snapshotValue: string;
  sourceRevision: string;
};
type ModulePacket = {
  source: string;
  compatSource: string;
  observed: { module: string; nativeCompat: string; babelCompat: string };
  nativeDto: NativeDto;
  sourceRevision: string;
};
type MapPacket = {
  source: string;
  observed: string;
  nativeDto: NativeDto;
  sourceRevision: string;
};
function read<T>(name: string, sha256: string): T {
  const bytes = fs.readFileSync(
    path.join(
      workspaceRoot,
      "tests/_fixtures/differential/jsx/child-values-6890-vdom-consumers",
      name,
    ),
  );
  assert.equal(createHash("sha256").update(bytes).digest("hex"), sha256);
  return JSON.parse(bytes.toString()) as T;
}
export const nativeVdomConsumers = {
  bundle: read<BundlePacket>(
    "native-vdom-bundle.json",
    "64f46a44722eca33f5d68b48624a1aefd318a99c2b0781c5edf5c58166188e21",
  ),
  module: read<ModulePacket>(
    "native-vdom-module.json",
    "4614a5b65f3ad9501f3ad3d4c3757d56733e9d0a3238c37815ed1833a2a2d35d",
  ),
  map: read<MapPacket>(
    "native-vdom-map.json",
    "eef1c71f8fed306768b83b7b51979ee4e43faaaba3549150f6f0ddf5566fdc23",
  ),
};
