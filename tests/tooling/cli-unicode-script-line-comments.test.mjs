import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  binaryRelativePath,
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../differential/build-receipt.ts";

const repository = fileURLToPath(new URL("../../", import.meta.url));
const root = new URL(
  "../_fixtures/differential/compiler/unicode-script-line-comments-11798/",
  import.meta.url,
);
const registration = JSON.parse(readFileSync(new URL("corpus.json", root), "utf8"));
const bytes = readFileSync(new URL(registration.path, root));
assert.equal(createHash("sha256").update(bytes).digest("hex"), registration.sha256);
const corpus = JSON.parse(bytes.toString());
const binary = join(repository, binaryRelativePath());
const receipt = JSON.parse(readFileSync(`${binary}.differential-build.json`, "utf8"));
validateBuildReceipt(receipt, expectedBuildIdentity(repository));

test("#11798 actual source CLI preserves all 36 authored script modules", () => {
  assert.equal(corpus.cases.length, 36);
  const observations = [];
  const project = mkdtempSync(join(tmpdir(), "vize-unicode-comments-11798-"));
  try {
    for (const row of corpus.cases) {
      writeFileSync(join(project, "Case.vue"), row.source);
      const output = join(project, "out", "Case.js");
      rmSync(join(project, "out"), { recursive: true, force: true });
      const result = spawnSync(
        binary,
        ["build", "Case.vue", "--no-config", "-o", "out", ...(row.ssr ? ["--ssr"] : [])],
        { cwd: project, env: { ...process.env, RAYON_NUM_THREADS: "1" }, timeout: 30_000 },
      );
      const code = existsSync(output) ? readFileSync(output, "utf8") : null;
      observations.push({
        id: row.id,
        source: row.source,
        exit: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        stdout: result.stdout?.toString("base64"),
        stderr: result.stderr?.toString("base64"),
        code,
      });
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      assert.equal(result.status, 0, result.stderr?.toString());
      assert.equal(code, row.code, row.id);
      assert.equal(readFileSync(join(project, "Case.vue"), "utf8"), row.source);
    }
  } finally {
    console.log("11798_CLI_RAW", JSON.stringify({ receipt, observations }));
    rmSync(project, { recursive: true, force: true });
  }
});
