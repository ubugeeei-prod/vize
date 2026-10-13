import assert from "node:assert/strict";
import test from "node:test";
import { projectOwnedCorsaNativeMainAfter } from "../../../tools/support/compat/github/owned-corsa-native-main-source.mjs";
import { createHash } from "node:crypto";

export async function runOwnedCargoOriginalRefusal({ actualView, copied, old, manifest, after }) {
  await test("actual owned Cargo tuple separately refuses the original helper before historical caller qualification", () => {
    const expectedLock = actualView.historicalCopied.find(
      ({ file }) => file === "Cargo.lock",
    ).sha256;
    const actualLock = copied.find(({ file }) => file === "Cargo.lock").sha256;
    let refusal;
    try {
      assert.equal(actualLock, expectedLock, "Cargo.lock");
    } catch (error) {
      refusal = error;
    }
    const incomingCopied = JSON.stringify(copied);
    const mainView = projectOwnedCorsaNativeMainAfter((side, file) => {
      assert.equal(side, "after");
      return after.get(file);
    });
    const oldMain = copied.map((entry) => {
      if (
        ![
          "crates/vize_canon/src/batch/type_checker/tests/recent_issues.rs",
          ".github/workflows/davinci-canon-scaling.yml",
        ].includes(entry.file)
      )
        return entry;
      const value = mainView(entry.file);
      return {
        ...entry,
        byteLength: value.body.length,
        sha256: createHash("sha256").update(value.body).digest("hex"),
        gitEntryHex: Buffer.from(`100644 blob ${value.blob}\t${entry.file}\0`).toString("hex"),
      };
    });
    assert.equal(JSON.stringify(copied), incomingCopied);
    assert.throws(() => old.validateLexicalSourceBodies(manifest, oldMain), {
      name: refusal.name,
      code: refusal.code,
      message: refusal.message,
      actual: refusal.actual,
      expected: refusal.expected,
      operator: refusal.operator,
      generatedMessage: refusal.generatedMessage,
    });
  });
}
