import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const root = new URL(
  "../_fixtures/differential/compiler/unicode-script-line-comments-11798/",
  import.meta.url,
);

void test("#11798 physical corpus pins 18 valid JavaScript sources in both targets", () => {
  const registration = JSON.parse(readFileSync(new URL("corpus.json", root), "utf8"));
  const bytes = readFileSync(new URL(registration.path, root));
  assert.equal(createHash("sha256").update(bytes).digest("hex"), registration.sha256);
  const corpus = JSON.parse(bytes.toString());
  assert.equal(corpus.cases.length, 36);
  const sources = new Set<string>();
  for (const row of corpus.cases) {
    assert.equal(createHash("sha256").update(row.source).digest("hex"), row.sourceSha256);
    assert.equal(row.source, `<script>${row.code}</script>`);
    assert.equal(row.wasm.script.code, row.code);
    assert.equal(row.moduleShape.defaultExportEnd, Buffer.byteLength(row.code));
    const code = Buffer.from(row.code);
    const rewritten = Buffer.concat([
      code.subarray(0, row.moduleShape.defaultExportStart),
      Buffer.from("const _sfc_main ="),
      code.subarray(row.moduleShape.defaultExportKeywordEnd),
    ]).toString();
    assert.equal(
      rewritten,
      `${row.code.startsWith("const label") ? "const label = '雪😀';\r\n" : ""}const _sfc_main = { label: '雪' };`,
    );
    const parsed = spawnSync(process.execPath, ["--input-type=module", "--check"], {
      input: row.code,
      encoding: "utf8",
    });
    assert.deepEqual(
      {
        status: parsed.status,
        signal: parsed.signal,
        error: parsed.error,
        stdout: parsed.stdout,
        stderr: parsed.stderr,
      },
      { status: 0, signal: null, error: undefined, stdout: "", stderr: "" },
      row.id,
    );
    sources.add(row.sourceSha256);
  }
  assert.equal(sources.size, 18);
});
