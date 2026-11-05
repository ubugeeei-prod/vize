import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import path from "node:path";

type NativePacket = { code: string; [key: string]: unknown };
export type PlainScriptPacket = {
  case: string;
  filename: string;
  source: string;
  sourceSha256: string;
  options: {
    filename: string;
    inlineTemplate: boolean;
    sourceMap: boolean;
    isProd: boolean;
  };
  native: NativePacket;
  official: unknown;
};

const constantControlsSource = "56ae19e666b9b8bf4ad88971d29d4c17c660506072a6f86f92a6b6d6464a41d0";

/**
 * #11865 adds project-watch metadata to the four preserved constant-controls
 * App packets. Specify that addition independently of the candidate packet;
 * code, maps, diagnostics and every other field remain owned by the existing
 * bracket/erasure reference and the final whole-packet comparison.
 */
export function plainScriptProjectMetadataReference<T extends NativePacket>(
  row: PlainScriptPacket,
  preserved: T,
  workspaceRoot: string,
): T {
  const out = structuredClone(preserved);
  if (row.case !== "constant-controls" || row.filename !== "src/App.vue") return out;

  assert.equal(row.sourceSha256, constantControlsSource);
  assert.equal(createHash("sha256").update(row.source).digest("hex"), constantControlsSource);
  assert.equal(typeof row.options.inlineTemplate, "boolean");
  assert.equal(typeof row.options.sourceMap, "boolean");
  assert.deepEqual(row.options, {
    filename: "src/App.vue",
    inlineTemplate: row.options.inlineTemplate,
    sourceMap: row.options.sourceMap,
    isProd: row.options.inlineTemplate,
  });
  assert.equal(Object.hasOwn(preserved, "dependencies"), false);
  assert.equal(Object.hasOwn(preserved, "typeProject"), false);
  assert.equal(path.posix.resolve(workspaceRoot), workspaceRoot);

  return {
    ...out,
    dependencies: [
      path.posix.join(workspaceRoot, "src", "tsconfig.json"),
      path.posix.join(workspaceRoot, "tsconfig.json"),
    ],
    typeProject: { root: path.posix.join(workspaceRoot, "tsconfig.json"), revision: "1" },
  };
}
