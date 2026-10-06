import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type {
  PublishedAuthority,
  PublishedReceipt,
} from "../../tooling/support/lsp/published-launch.ts";
import { driverRoot, sha256 } from "./warm-type-backed-source.ts";

/** Rerun the root's frozen tool-derived whole-tree relation, not claimed booleans. */
export function verifyReleaseBridge(
  authority: PublishedAuthority,
  output: string,
): PublishedReceipt["versionBridge"] {
  const projection = authority.versionBridge.projection;
  assert.equal(projection.schema, "vize.release.source-cut-version-bridge");
  assert.equal(projection.version, 1);
  assert.deepEqual(projection.sourceCut, {
    sha: authority.sourceCut.sha,
    tree: authority.sourceCut.tree,
    version: authority.sourceCut.version,
  });
  assert.deepEqual(projection.tagHead, authority.tagHead);
  for (const source of [projection.sourceCut, projection.tagHead]) {
    assert.match(source.sha, /^[a-f0-9]{40}$/u);
    assert.match(source.tree, /^[a-f0-9]{40}$/u);
  }
  for (const digest of [
    projection.recipeSha256,
    projection.allPathManifestSha256,
    projection.changedPathManifestSha256,
    authority.versionBridge.sha256,
  ])
    assert.match(digest, /^[a-f0-9]{64}$/u);
  for (const count of [projection.changedPathCount, projection.unchangedPathCount])
    assert.ok(Number.isSafeInteger(count) && count > 0);
  assert.equal(sha256(JSON.stringify(projection)), authority.versionBridge.sha256);
  assert.ok(Number.isSafeInteger(authority.releasePr) && authority.releasePr > 7811);
  const script = fileURLToPath(new URL("./warm-type-backed-release-bridge.py", import.meta.url));
  const recipe = fileURLToPath(new URL("./warm-type-backed-release-bridge.json", import.meta.url));
  assert.equal(
    sha256(fs.readFileSync(script)),
    "8f72189ee25b56f64413f49ce717b1c29750c78d3ac41d65b1c85f89a5b8a767",
    "the independently reviewed verifier bytes are frozen",
  );
  assert.equal(
    sha256(fs.readFileSync(recipe)),
    "6887483d6ddfbb8f7a46e3b2dae85e85d14015c670299da46190896f199f4093",
    "the official scoped recipe cannot be caller supplied",
  );
  assert.equal(sha256(fs.readFileSync(recipe)), projection.recipeSha256);
  const receiptPath = path.join(output, "official-version-bridge.json");
  const args = [
    script,
    driverRoot,
    authority.sourceCut.sha,
    authority.tagHead.sha,
    authority.sourceCut.version,
    authority.tagHead.version,
    recipe,
    receiptPath,
    "--pr",
    String(authority.releasePr),
  ];
  const result = spawnSync("python3", args, { cwd: driverRoot, maxBuffer: 64 * 1024 * 1024 });
  fs.writeFileSync(path.join(output, "official-version-bridge.stdout"), result.stdout);
  fs.writeFileSync(path.join(output, "official-version-bridge.stderr"), result.stderr);
  fs.writeFileSync(
    path.join(output, "official-version-bridge-command.json"),
    `${JSON.stringify(
      {
        command: ["python3", ...args],
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        scriptSha256: sha256(fs.readFileSync(script)),
        stdoutSha256: sha256(result.stdout),
        stderrSha256: sha256(result.stderr),
      },
      null,
      2,
    )}\n`,
  );
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, result.stderr.toString());
  const bytes = fs.readFileSync(receiptPath);
  const bridge = JSON.parse(bytes.toString("utf8"));
  assert.equal(bridge.status, "CANDIDATE_TREE_RELATION_VERIFIED");
  assert.deepEqual(bridge.authorityProjection, projection);
  assert.equal(bridge.authorityProjectionSha256, authority.versionBridge.sha256);
  return { path: receiptPath, sha256: sha256(bytes) };
}
