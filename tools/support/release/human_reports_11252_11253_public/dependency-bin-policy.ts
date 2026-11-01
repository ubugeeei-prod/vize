import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import type { Request } from "./model.ts";
import { bytes, gitBlob, sha256 } from "./value.ts";

export const binPolicyPath =
  "tools/support/release/human_reports_11252_11253_public/dependency-bin-policy.json.txt";
export interface BinPolicy {
  path: typeof binPolicyPath;
  gitBlob: string;
  sha256: string;
  bytes: number;
  rawBase64: string;
}
/** The expected blob is supplied by the reviewed MAIN controller inventory,
 * whose whole remote Git closure is authenticated before any install. A
 * capture's own digest can never establish this policy's authority. */
export function bindBinPolicy(raw: Buffer, reviewed: Request["reviewedController"]): BinPolicy {
  const rows = reviewed.files.filter((row) => row.path === binPolicyPath);
  assert.equal(rows.length, 1, "reviewed controller must own the input policy");
  const row = rows[0]!;
  assert.equal(raw.length, row.bytes);
  assert.equal(sha256(raw), row.sha256);
  assert.equal(gitBlob(raw), row.gitBlob);
  return {
    path: binPolicyPath,
    gitBlob: row.gitBlob,
    sha256: row.sha256,
    bytes: raw.length,
    rawBase64: raw.toString("base64"),
  };
}
export function controllerBinPolicy(reviewed: Request["reviewedController"]): BinPolicy {
  return bindBinPolicy(
    readFileSync(new URL("./dependency-bin-policy.json.txt", import.meta.url)),
    reviewed,
  );
}
export function assertRecordedBinPolicy(
  policy: BinPolicy,
  reviewed: Request["reviewedController"],
): void {
  assert.equal(policy.path, binPolicyPath);
  assert.deepEqual(bindBinPolicy(bytes(policy.rawBase64), reviewed), policy);
}
