import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

import { repoRoot } from "./realworld-patch.ts";

// Official @vue/reactivity 3.6.0-beta.10 package, authenticated against the
// frozen lock's SHA512 SRI. These are declaration facts, not a Vize capture.
export function createVueInlayContract() {
  const declaration = fs.realpathSync(
    path.join(repoRoot, "node_modules/@vue/reactivity/dist/reactivity.d.ts"),
  );
  const metadata = JSON.parse(
    fs.readFileSync(path.join(path.dirname(declaration), "../package.json"), "utf8"),
  );
  assert.equal(metadata.name, "@vue/reactivity");
  assert.equal(metadata.version, "3.6.0-beta.10");
  const bytes = fs.readFileSync(declaration);
  assert.equal(
    createHash("sha256").update(bytes).digest("hex"),
    "b4ad4832d51e98465a13e35d1024957eeb87a70b6fe4448409e471998a139644",
    "re-derive the complete native contract if the pinned declaration changes",
  );
  const lines = bytes.toString("utf8").split("\n");
  assert.equal(lines[489], "export interface Ref<T = any, S = T> {");
  assert.equal(lines[187], "export interface ComputedRef<T = any> extends BaseComputedRef<T> {");
  const uri = pathToFileURL(declaration).href;
  const location = (line: number, width: number) => ({
    uri,
    range: { start: { line, character: 17 }, end: { line, character: 17 + width } },
  });
  // Native 7.0.2 internal/ls/inlay_hints.go:231-252,311-320,335-346:
  // checker types, alias-aware label parts, declaration-name end, kind Type,
  // paddingLeft true. Ref's getter/setter generic arguments both infer number;
  // computed's numeric getter infers ComputedRef<number>. No synthetic tooltip.
  return {
    declaration,
    declarationBytes: bytes,
    expected: [
      {
        kind: 1,
        label: [
          { value: ": " },
          { value: "Ref", location: location(489, 3) },
          { value: "<" },
          { value: "number" },
          { value: "," },
          { value: "number" },
          { value: ">" },
        ],
        paddingLeft: true,
        position: { line: 3, character: 16 },
      },
      {
        kind: 1,
        label: [
          { value: ": " },
          { value: "ComputedRef", location: location(187, 11) },
          { value: "<" },
          { value: "number" },
          { value: ">" },
        ],
        paddingLeft: true,
        position: { line: 4, character: 13 },
      },
    ],
  };
}
