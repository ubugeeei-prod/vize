import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { sha256 } from "../../differential/harness.mjs";
import { loadFocusCases } from "../../differential/focus-history.ts";
import { loadFocusCurrentOracle } from "../../differential/focus-history-current-oracle.ts";
import { currentFocusSummary } from "../../differential/focus-history-current-report.ts";

export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
export const fixtures = loadFocusCases(root);

export function attempt(bytes: Buffer) {
  return {
    stdoutBase64: bytes.toString("base64"),
    stdoutSha256: sha256(bytes),
    stderrBase64: "",
    stderrSha256: sha256(Buffer.alloc(0)),
    exitStatus: 0,
    signal: null,
    processError: null,
  };
}

/** Deliberately synthetic in-memory envelopes exercise validators only.
 * No native executable runs, no expected capture is written, and the complete
 * authority remains the independently reviewed immutable legacy bytes. */
export function syntheticCurrentCapture() {
  const historical = loadFocusCurrentOracle(root);
  const receipt = structuredClone(historical.buildReceipt);
  receipt.source.sourceRevision = "a".repeat(40);
  const rows = historical.rows.map((original: any) => ({
    id: original.id,
    inputBase64: original.inputBase64,
    inputSha256: original.inputSha256,
    sourceSha256: original.sourceSha256,
    legacy: structuredClone(original.legacy),
    native: {
      state: "handled",
      argv: ["--native"],
      attempts: original.legacy.attempts.map((raw: any) =>
        attempt(
          Buffer.from(
            `${JSON.stringify({
              state: "handled",
              observation: Buffer.from(raw.stdoutBase64, "base64").toString("utf8"),
            })}\n`,
          ),
        ),
      ),
    },
    comparison: { state: "not-compared", reason: "capture-only" },
  }));
  return {
    schema: "vize.focus-history.capture",
    version: 2,
    acceptance: "unreviewed",
    sourceRevision: receipt.source.sourceRevision,
    fixtureSha256: historical.fixtureSha256,
    buildReceipt: receipt,
    rows,
    summary: currentFocusSummary(rows),
  };
}

export function replaceWhole(lane: any, before: string, after: string) {
  for (const raw of lane.attempts) {
    const bytes = Buffer.from(raw.stdoutBase64, "base64");
    const envelope = lane.argv[0] === "--native" ? JSON.parse(bytes.toString()) : null;
    const body = envelope ? envelope.observation : bytes.toString();
    assert(body.includes(before), `meaningful whole-output mutation: ${before}`);
    const changed = body.replaceAll(before, after);
    const output = envelope
      ? `${JSON.stringify({ ...envelope, observation: changed })}\n`
      : changed;
    const replacement = attempt(Buffer.from(output));
    Object.assign(raw, replacement);
  }
}
