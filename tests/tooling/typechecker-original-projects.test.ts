import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { sha256 } from "../differential/harness.mjs";
import { loadTypecheckerManifest } from "../differential/typechecker.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const directory = path.join(root, "tests/_fixtures/differential/typechecker");
const loaded = loadTypecheckerManifest(path.join(directory, "manifest.json"));

test("new required projects preserve original authored input bytes and complete tuple assertions", () => {
  for (const name of ["typed-import-meta", "slot-outlet-key"]) {
    const base = path.join(directory, name);
    const proof = JSON.parse(fs.readFileSync(path.join(base, "input-provenance.json"), "utf8"));
    const pack = JSON.parse(fs.readFileSync(path.join(base, "cases.json"), "utf8"));
    const original = fs.readFileSync(path.join(base, proof.originalTestSource.carrier));
    const excerpt = proof.projectHelperSource.excerpt;
    const helper = fs.readFileSync(path.join(base, excerpt.carrier));
    assert.equal(sha256(original), proof.originalTestSource.sha256);
    assert.equal(sha256(helper), excerpt.sha256);
    assert.equal(helper.length, excerpt.utf8End - excerpt.utf8Start);
    assert.equal(pack.sourceRevision, proof.regressionCommit);
    assert.equal(pack.regressionCommit, proof.regressionCommit);
    assert(original.toString("utf8").includes(`fn ${proof.originalTest}()`));
    assert(helper.toString("utf8").includes('1 => "error"'));
    assert(helper.toString("utf8").includes("snapshot.sort();"));
    assert(!helper.toString("utf8").includes(".filter("));
    for (const input of proof.inputs) {
      const bytes = fs.readFileSync(path.join(base, input.source));
      assert.equal(sha256(bytes), input.sha256);
      const source = input.file === "tsconfig.json" ? helper : original;
      const offset =
        input.file === "tsconfig.json"
          ? proof.configLiteralUtf8Start - excerpt.utf8Start
          : input.originalLiteralUtf8Start;
      assert.deepEqual(bytes, source.subarray(offset, offset + bytes.length));
    }
    const planned = loaded.cases.find((fixture) => fixture.pack === name);
    assert(planned);
    assert.equal(planned.sourceRevision, proof.regressionCommit);
    assert.deepEqual(planned.diagnostics, pack.cases[0].diagnostics);
    const text = original.toString("utf8");
    if (name === "typed-import-meta") {
      const tuple =
        /Some\(vec!\[\(\s*vize_l0::String::from\("([^"]+)"\),\s*Some\((\d+)\),\s*vize_l0::String::from\("(\d+):(\d+):error ([^"]+)"\),\s*\)\]\)/.exec(
          text,
        );
      assert(tuple);
      assert.deepEqual(planned.diagnostics, [
        {
          file: tuple[1],
          line: Number(tuple[3]),
          column: Number(tuple[4]),
          severity: 1,
          code: Number(tuple[2]),
          message: tuple[5],
        },
      ]);
    } else {
      assert(text.includes("assert_eq!(snapshot, Some(Vec::new()));"));
      assert.deepEqual(planned.diagnostics, []);
    }
    assert.equal(proof.execution, "pending fresh required T1 source-built Actions");
    assert.equal(proof.native, "unsupported");
  }
});
