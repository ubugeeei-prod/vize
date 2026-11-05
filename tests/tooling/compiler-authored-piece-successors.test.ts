import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { test } from "node:test";
import {
  assertAuthoredPieceCompilerSuccessor,
  authoredPieceCompilerSuccessor,
  authoredPieceRouterProps,
  verifyAuthoredPieceCompilerPair,
  type AuthoredPieceCompilerCase,
} from "../_helpers/compiler-authored-piece-successors.ts";
import { compilerPatchSuccessor } from "../_helpers/compiler-patch-successors.ts";

const cases: AuthoredPieceCompilerCase[] = [
  "create-vue-clean",
  "create-vue-broken",
  "create-vue-formatted",
  "vue-router-clean",
  "vue-router-broken",
];
const pins = (id: AuthoredPieceCompilerCase) => {
  if (id === "create-vue-formatted") {
    return { codeSha256: "17e394dd258667ffaae5ae7bc92357ff703fcb15f26854244404c8587f47d07b" };
  }
  const { review } = compilerPatchSuccessor(id);
  return { packetSha256: review.previousSha256, codeSha256: review.previousCodeSha256 };
};

test("authored-piece erasure preserves the frozen packets in both directions and all JS syntax", () => {
  const capture = JSON.parse(
    fs.readFileSync(
      new URL(
        "../_fixtures/compiler-authored-piece-successors-11698/capture.json",
        import.meta.url,
      ),
      "utf8",
    ),
  ) as { rows: { id: AuthoredPieceCompilerCase; sourceSha256: string }[] };
  assert.deepEqual(
    capture.rows.map((row) => row.id),
    cases,
  );
  for (const id of cases) {
    const { previous, output, raw } = authoredPieceCompilerSuccessor(id);
    verifyAuthoredPieceCompilerPair(id, previous, output);
    assertAuthoredPieceCompilerSuccessor(id, raw, pins(id));
    assert.equal(
      capture.rows.find((row) => row.id === id)?.sourceSha256,
      id === "create-vue-formatted"
        ? "f6c836cc1aa667f33f324274df259e30580183194259abe0546efd5f552b8728"
        : compilerPatchSuccessor(id).review.inputSha256,
    );
    const parsed = spawnSync(process.execPath, ["--input-type=module", "--check"], {
      input: output.code,
      encoding: "utf8",
    });
    assert.equal(parsed.status, 0, parsed.stderr);
    if (id.startsWith("vue-router")) {
      assert.equal(output.code.split(authoredPieceRouterProps).length, 2);
    }
  }
});

test("authored-piece successors reject changed, missing and extra packet fields and every raw byte", () => {
  for (const id of cases) {
    const { previous, output, raw } = authoredPieceCompilerSuccessor(id);
    for (const field of Object.keys(output)) {
      const changed = { ...output, [field]: "unrelated" };
      const missing = { ...output } as Partial<typeof output>;
      delete missing[field as keyof typeof output];
      for (const mutation of [changed, missing, { ...output, unknownFacet: true }]) {
        assert.throws(() =>
          verifyAuthoredPieceCompilerPair(id, previous, mutation as typeof output),
        );
        assert.throws(() =>
          assertAuthoredPieceCompilerSuccessor(id, JSON.stringify(mutation, null, 2), pins(id)),
        );
      }
    }
    for (const altered of [raw + "\n", JSON.stringify(output), raw.replace("__name", "__NAME")]) {
      assert.throws(() => assertAuthoredPieceCompilerSuccessor(id, altered, pins(id)));
    }
    assert.throws(() =>
      assertAuthoredPieceCompilerSuccessor(id, raw, { codeSha256: "0".repeat(64) }),
    );
  }
});
