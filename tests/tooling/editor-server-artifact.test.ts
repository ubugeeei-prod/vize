import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  sha256,
  verifyArtifact,
  verifyHost,
  verifyScorecard,
  writeArtifact,
} from "../editor-conformance/server-artifact.ts";

test("editor artifacts reject another commit and altered server bytes", async (context) => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), "vize-artifact-"));
  context.after(() => fs.rm(directory, { recursive: true, force: true }));
  const binary = path.join(directory, "vize");
  const receipt = path.join(directory, "server.json");
  const source = "a".repeat(40);
  await fs.writeFile(binary, "source build");
  await writeArtifact(binary, receipt, source);
  await verifyArtifact(binary, receipt, source);
  await assert.rejects(verifyArtifact(binary, receipt, "b".repeat(40)));
  await fs.appendFile(binary, "changed");
  await assert.rejects(verifyArtifact(binary, receipt, source));
});

test("host qualification requires executed cases, native identity and intact packets", async (context) => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), "vize-host-receipt-"));
  context.after(() => fs.rm(directory, { recursive: true, force: true }));
  const binary = path.join(directory, "vize");
  const artifact = path.join(directory, "server.json");
  const receipt = path.join(directory, "receipt.json");
  const transcript = path.join(directory, "transcript.jsonl");
  const sourceSha = "a".repeat(40);
  await fs.writeFile(binary, "source build");
  await fs.writeFile(transcript, '{"dir":"c2s","msg":{"method":"initialize"}}\n');
  await writeArtifact(binary, artifact, sourceSha);
  const valid = {
    passed: true,
    sourceSha,
    binary: { sha256: await sha256(binary), nativePid: 123 },
    transcript: { path: transcript, sha256: await sha256(transcript) },
    cases: [{ phase: "executed" }],
  };
  const check = async (value: unknown) => {
    await fs.writeFile(receipt, JSON.stringify(value));
    await verifyHost(artifact, receipt, sourceSha);
  };
  await check(valid);
  for (const change of [
    { passed: false },
    { sourceSha: "b".repeat(40) },
    { cases: [] },
    { binary: { ...valid.binary, nativePid: null } },
    { binary: { ...valid.binary, sha256: "b".repeat(64) } },
  ])
    await assert.rejects(check({ ...valid, ...change }));
  await fs.appendFile(transcript, '{"extra":"unrecorded"}\n');
  await assert.rejects(check(valid));
});

test("full packaged host results bind the actually selected server to the source artifact", async (context) => {
  const directory = await fs.mkdtemp(path.join(os.tmpdir(), "vize-scorecard-"));
  context.after(() => fs.rm(directory, { recursive: true, force: true }));
  const binary = path.join(directory, "vize");
  const artifact = path.join(directory, "server.json");
  const result = path.join(directory, "host-result.json");
  const sourceSha = "a".repeat(40);
  await fs.writeFile(binary, "source build");
  await writeArtifact(binary, artifact, sourceSha);
  await fs.writeFile(
    result,
    JSON.stringify({ serverInfo: { path: binary, source: "configured", status: "ready" } }),
  );
  await verifyScorecard(artifact, result, sourceSha);
  const receipt = JSON.parse(
    await fs.readFile(path.join(directory, "scorecard-receipt.json"), "utf8"),
  );
  assert.equal(receipt.passed, true);
  assert.equal(receipt.sourceSha, sourceSha);
  assert.equal(receipt.result.sha256, await sha256(result));
  await assert.rejects(verifyScorecard(artifact, result, "b".repeat(40)));
  await fs.appendFile(binary, "altered");
  await assert.rejects(verifyScorecard(artifact, result, sourceSha));
});
