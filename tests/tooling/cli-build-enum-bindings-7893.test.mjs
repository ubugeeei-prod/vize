import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  binaryRelativePath,
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../differential/build-receipt.mjs";

const repository = fileURLToPath(new URL("../../", import.meta.url));
const originalPath = new URL(
  "../_fixtures/differential/compiler/enum-template-bindings-7893/Badge.vue.txt",
  import.meta.url,
);
const bytes = readFileSync(originalPath);
const source = bytes.toString("utf8");
const originalSha256 = "02a9907355de1b26720b9a7cd847f81c42626a3600b2663c334fa64fb89db113";
const binary = join(repository, binaryRelativePath());
const receipt = JSON.parse(readFileSync(`${binary}.differential-build.json`, "utf8"));
validateBuildReceipt(receipt, expectedBuildIdentity(repository));

void test("#7893 original CLI builds and complete Vue runtime observations agree", () => {
  assert.equal(bytes.length, 269);
  assert.equal(createHash("sha256").update(bytes).digest("hex"), originalSha256);
  const evidence = join(repository, "target/differential/enum-template-bindings-7893");
  mkdirSync(evidence, { recursive: true });
  const rows = [];
  try {
    for (const { lane, flags } of [
      { lane: "dom", flags: [] },
      { lane: "ssr", flags: ["--ssr"] },
      { lane: "vapor", flags: ["--vapor"] },
    ]) {
      const project = mkdtempSync(join(tmpdir(), "vize-enum-7893-"));
      const row = { lane, source, originalSha256, receipt, status: "RUNNING" };
      rows.push(row);
      try {
        writeFileSync(join(project, "Badge.vue"), bytes);
        const args = ["build", "Badge.vue", "-o", "out", ...flags];
        const built = spawnSync(binary, args, {
          cwd: project,
          env: { ...process.env, RAYON_NUM_THREADS: "1" },
          timeout: 60_000,
          maxBuffer: 8 * 1024 * 1024,
        });
        const output = join(project, "out/Badge.js");
        const code = existsSync(output) ? readFileSync(output, "utf8") : null;
        Object.assign(row, {
          args,
          cwd: project,
          exit: built.status,
          signal: built.signal,
          error: built.error?.message ?? null,
          stdoutBase64: (built.stdout ?? Buffer.alloc(0)).toString("base64"),
          stderrBase64: (built.stderr ?? Buffer.alloc(0)).toString("base64"),
          code,
        });
        assert.equal(built.error, undefined);
        assert.equal(built.signal, null);
        assert.equal(built.status, 0, built.stderr?.toString());
        assert.deepEqual(built.stdout, Buffer.alloc(0));
        assert.deepEqual(readFileSync(join(project, "Badge.vue")), bytes);
        assert.equal(typeof code, "string");
        const runtime = spawnSync(
          process.execPath,
          [fileURLToPath(new URL("./support/enum-template-runtime-7893.mjs", import.meta.url))],
          {
            input: JSON.stringify({ lane, source, code }),
            timeout: 90_000,
            maxBuffer: 16 * 1024 * 1024,
          },
        );
        Object.assign(row, {
          runtimeExit: runtime.status,
          runtimeSignal: runtime.signal,
          runtimeError: runtime.error?.message ?? null,
          runtimeStdoutBase64: (runtime.stdout ?? Buffer.alloc(0)).toString("base64"),
          runtimeStderrBase64: (runtime.stderr ?? Buffer.alloc(0)).toString("base64"),
        });
        if (runtime.stdout?.length) row.runtime = JSON.parse(runtime.stdout);
        assert.equal(runtime.error, undefined);
        assert.equal(runtime.signal, null);
        assert.equal(runtime.status, 0, runtime.stderr?.toString());
        assert.deepEqual(row.runtime.actual, row.runtime.reference);
        row.status = "PASS";
      } catch (error) {
        Object.assign(row, {
          status: "FAIL",
          failure: { message: error.message, stack: error.stack },
        });
      } finally {
        writeFileSync(join(evidence, "whole-observations.json"), JSON.stringify(rows, null, 2));
        rmSync(project, { recursive: true, force: true });
      }
    }
    assert.deepEqual(
      rows.map(({ lane, status }) => ({ lane, status })),
      [
        { lane: "dom", status: "PASS" },
        { lane: "ssr", status: "PASS" },
        { lane: "vapor", status: "PASS" },
      ],
    );
  } finally {
    writeFileSync(join(evidence, "whole-observations.json"), JSON.stringify(rows, null, 2));
  }
});
