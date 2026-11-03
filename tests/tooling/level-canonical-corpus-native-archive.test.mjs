import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  collectorHarness,
  validateNativeSource,
} from "../../tools/support/compat/github/canonical-corpus-native-walk.ts";
import { gitObjectId } from "../../tools/support/compat/github/canonical-corpus-inventory.mjs";

await test("native archive source/head/toolchain/binary and entire raw-vector custody refuses replacements", () => {
  const temporary = mkdtempSync(join(tmpdir(), "canonical-native-archive-"));
  const root = join(temporary, "inputs");
  mkdirSync(root);
  writeFileSync(join(root, "Original.vue"), "<template>whole original</template>\r\n");
  const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
  const git = (...args) => execFileSync("git", args, { encoding: "utf8" }).trim();
  const identity = {
    sha: git("rev-parse", "HEAD"),
    tree: git("rev-parse", "HEAD^{tree}"),
    runId: 12,
    attempt: 1,
  };
  try {
    const source = readFileSync("tests/davinci_test_support/src/corpus.rs");
    const { body, harness } = collectorHarness(source);
    writeFileSync(join(temporary, "native-collector-source.rs"), source);
    writeFileSync(join(temporary, "native-collector.rs"), harness);
    execFileSync("rustc", [
      "--edition=2024",
      join(temporary, "native-collector.rs"),
      "-o",
      join(temporary, "native-collector"),
    ]);
    const toolchain = execFileSync("rustc", ["-vV"]);
    const vector = execFileSync(join(temporary, "native-collector"), [root]);
    writeFileSync(join(temporary, "native-rustc.txt"), toolchain);
    writeFileSync(join(temporary, "native-vector.nul"), vector);
    const original = [
      [
        "Original.vue",
        hash(readFileSync(join(root, "Original.vue"))),
        readFileSync(join(root, "Original.vue")).length,
        gitObjectId("blob", readFileSync(join(root, "Original.vue"))),
      ],
    ];
    writeFileSync(join(temporary, "original-kernel-vector.nul"), vector);
    writeFileSync(join(temporary, "original-kernel-files.json"), JSON.stringify(original));
    const receipt = {
      schema: "vize.canonical-native-walk",
      version: 1,
      ...identity,
      sourceSha256: hash(source),
      bodySha256: hash(body),
      harnessSha256: hash(harness),
      binarySha256: hash(readFileSync(join(temporary, "native-collector"))),
      toolchainSha256: hash(toolchain),
      vectorSha256: hash(vector),
      files: 1,
      traversal: "finite-physical-edges",
      originalVectorSha256: hash(vector),
      originalFilesSha256: hash(JSON.stringify(original)),
    };
    const receiptPath = join(temporary, "native-walk.json");
    writeFileSync(receiptPath, JSON.stringify(receipt));
    assert.deepEqual(validateNativeSource(temporary, identity).paths, ["Original.vue"]);
    for (const key of ["traversal", "originalVectorSha256", "originalFilesSha256"]) {
      const omitted = { ...receipt };
      delete omitted[key];
      writeFileSync(receiptPath, JSON.stringify(omitted));
      assert.throws(() => validateNativeSource(temporary, identity), `Omitted ${key}`);
    }
    writeFileSync(receiptPath, JSON.stringify(receipt));
    for (const name of [
      "native-collector-source.rs",
      "native-collector.rs",
      "native-collector",
      "native-rustc.txt",
      "native-vector.nul",
      "original-kernel-vector.nul",
      "original-kernel-files.json",
    ]) {
      const path = join(temporary, name);
      const original = readFileSync(path);
      writeFileSync(path, Buffer.concat([original, Buffer.from("changed")]));
      assert.throws(() => validateNativeSource(temporary, identity), `Changed ${name}`);
      writeFileSync(path, original);
    }
    for (const key of [
      "sha",
      "tree",
      "runId",
      "attempt",
      "sourceSha256",
      "bodySha256",
      "harnessSha256",
      "binarySha256",
      "toolchainSha256",
      "vectorSha256",
      "files",
      "traversal",
      "originalVectorSha256",
      "originalFilesSha256",
    ]) {
      writeFileSync(receiptPath, JSON.stringify({ ...receipt, [key]: "foreign" }));
      assert.throws(() => validateNativeSource(temporary, identity), `Foreign ${key}`);
    }
    writeFileSync(receiptPath, JSON.stringify(receipt));
    assert.deepEqual(validateNativeSource(temporary, identity).paths, ["Original.vue"]);
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});
