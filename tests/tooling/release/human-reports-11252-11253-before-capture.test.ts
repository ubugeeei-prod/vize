import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { writeZip, zipEntries } from "../../../tools/support/archive/zip.ts";
import {
  bindCapturePolicy,
  capturePolicyPath,
  decodeCapturedArchive,
  authenticateCapturedBefore,
} from "../../../tools/support/release/human_reports_11252_11253_public/public-before-capture.ts";
import {
  requireCaptureJudgment,
  type CapturePolicyEntry,
} from "../../../tools/support/release/human_reports_11252_11253_public/before-capture-record.ts";
import { loadOriginalFixture } from "../../../tools/support/release/human_reports_11252_11253_public/fixtures.ts";
import {
  afterPlan,
  requiredFixPaths,
} from "../../../tools/support/release/human_reports_11252_11253_public/after-plan.ts";
import {
  requireComplete,
  requireCaptureReady,
} from "../../../tools/support/release/human_reports_11252_11253_public/recipes.ts";
import {
  gitBlob,
  sha256,
} from "../../../tools/support/release/human_reports_11252_11253_public/value.ts";
import type {
  Bundle,
  Request,
  ReleaseAuthority,
  UnqualifiedCapture,
} from "../../../tools/support/release/human_reports_11252_11253_public/model.ts";

const raw = readFileSync(
  new URL(
    "../../../tools/support/release/human_reports_11252_11253_public/before-capture-policy.json.txt",
    import.meta.url,
  ),
);
const policy = JSON.parse(raw.toString()) as {
  target: Record<string, string>;
  roles: CapturePolicyEntry[];
};
function fixture(index = 0, body = raw) {
  const entry = policy.roles[index]!;
  const request: Request = {
    schema: "vize.humanReports11252_11253.request.v1",
    mode: "UNQUALIFIED_AFTER",
    role: entry.role,
    fixture: loadOriginalFixture(entry.role),
    reviewedController: {
      head: "f".repeat(40),
      files: [
        {
          path: capturePolicyPath,
          bytes: body.length,
          sha256: sha256(body),
          gitBlob: gitBlob(body),
        },
      ],
    },
    release: {
      issue: entry.issue,
      phase: "after",
      version: policy.target.version!,
      releaseRun: policy.target.releaseRun!,
      capsuleBlob: policy.target.capsuleBlob!,
      fixes: [
        {
          issue: entry.issue,
          pull: entry.issue === 11252 ? 11257 : 11258,
          originalHead: "e".repeat(40),
          merge: entry.ownerFixMerge,
          paths: [...requiredFixPaths[entry.issue]],
        },
      ],
    },
    commands: afterPlan(entry.role, policy.target.version!).map((p) => p.command),
    sourceAdmission: {
      commit: "e".repeat(40),
      run: "1",
      receiptBlob: "d".repeat(40),
      receiptSha256: "c".repeat(64),
    },
    oracleOwnerReceipt: null,
    publicBefore: null,
    capturedBefore: entry.reference,
  };
  const authority: ReleaseAuthority = {
    version: policy.target.version!,
    run: policy.target.releaseRun!,
    workflowHead: policy.target.workflowHead!,
    source: policy.target.source!,
    candidate: policy.target.candidate!,
    capsule: {},
    sdk: [],
    observations: [],
    produced: null,
  };
  return { entry, request, authority };
}
test("reviewed source DATA admits each separate raw reference without a strict before bundle or target oracle", () => {
  for (const index of [0, 1]) {
    const f = fixture(index);
    assert.deepEqual(bindCapturePolicy(raw, f.request, f.authority), f.entry);
    requireCaptureReady(f.request);
    assert.throws(() => requireComplete({ ...f.request, mode: "ACCEPT_AFTER" }));
    assert(f.request.commands.every((c) => c.oracle === null));
    assert.equal(f.request.publicBefore, null);
  }
});
test("selected-role independent review is mandatory while an unselected missing review never blocks the other role", () => {
  const changed = structuredClone(policy);
  changed.roles[1]!.judgment = null;
  const body = Buffer.from(JSON.stringify(changed));
  const first = fixture(0, body);
  assert.equal(bindCapturePolicy(body, first.request, first.authority).issue, 11252);
  const second = fixture(1, body);
  assert.throws(
    () => bindCapturePolicy(body, second.request, second.authority),
    /selected role lacks/u,
  );
});
test("a self-hash, missing source seed, wrong target or replaced reference refuses before transport or install", async () => {
  const f = fixture();
  for (const mutate of [
    (r: Request) => {
      r.reviewedController.files = [];
    },
    (r: Request) => {
      r.reviewedController.files[0]!.gitBlob = "0".repeat(40);
    },
    (r: Request) => {
      r.capturedBefore = { ...r.capturedBefore!, captureMemberSha256: "0".repeat(64) };
    },
    (r: Request) => {
      r.mode = "ACCEPT_AFTER";
    },
  ]) {
    const r = structuredClone(f.request);
    mutate(r);
    assert.throws(() => bindCapturePolicy(raw, r, f.authority));
    let calls = 0;
    await assert.rejects(
      authenticateCapturedBefore(
        r,
        f.authority,
        async () => {
          calls++;
          return {};
        },
        () => {},
        async () => {
          calls++;
          throw Error("transport reached");
        },
      ),
    );
    assert.equal(calls, 0);
  }
  assert.throws(() => bindCapturePolicy(raw, f.request, { ...f.authority, version: "0.629.0" }));
});
test("source-owner review is before-only, complete and role/fix/fixture bound; output authority and unreviewed placeholders refuse", () => {
  const e = policy.roles[0]!;
  requireCaptureJudgment(e);
  for (const mutate of [
    (v: CapturePolicyEntry) => {
      v.judgment = null;
    },
    (v: CapturePolicyEntry) => {
      v.judgment!.knownBugReproduction = "UNRESOLVED";
    },
    (v: CapturePolicyEntry) => {
      v.judgment!.completeOriginalAndCompanionPacketsReviewed = false;
    },
    (v: CapturePolicyEntry) => {
      v.judgment!.ownerFixMerge = "0".repeat(40);
    },
    (v: CapturePolicyEntry) => {
      v.judgment!.fixtureReceiptSha256 = "0".repeat(64);
    },
    (v: CapturePolicyEntry) => {
      v.judgment!.afterOutputsSupplied = true;
    },
    (v: CapturePolicyEntry) => {
      v.judgment!.sourceAdaptedOutputsTransferred = true;
    },
    (v: CapturePolicyEntry) => {
      v.judgment!.accepted = true;
    },
    (v: CapturePolicyEntry) => {
      v.judgment!.oracles = [];
    },
  ]) {
    const v = structuredClone(e);
    mutate(v);
    assert.throws(() => requireCaptureJudgment(v));
  }
});
test("an API run-source mismatch refuses before archive transport or installation", async () => {
  const f = fixture();
  let reads = 0,
    downloads = 0;
  await assert.rejects(
    authenticateCapturedBefore(
      f.request,
      f.authority,
      async () => {
        reads++;
        return {
          id: Number(f.entry.reference.controllerRun),
          repository: { full_name: "ubugeeei-prod/vize" },
          head_sha: "0".repeat(40),
        };
      },
      () => {},
      async () => {
        downloads++;
        throw Error("unexpected archive fetch");
      },
    ),
  );
  assert.equal(reads, 1);
  assert.equal(downloads, 0);
});
function archiveFixture(files: Array<{ name: string; data: string }>) {
  const archive = writeZip(files),
    entries = zipEntries(archive),
    entry = structuredClone(policy.roles[0]!);
  entry.reference.archiveDigest = "sha256:" + sha256(archive);
  entry.reference.captureMemberSha256 = sha256(Buffer.from("inert capture bytes; never evaluated"));
  entry.artifact = {
    ...entry.artifact,
    compressedBytes: archive.length,
    entries: entries.length,
    decodedBytes: entries.reduce((n, e) => n + e.size, 0),
  };
  return { archive, entry };
}
const safe = [
  { name: "unqualified-capture.json", data: "inert capture bytes; never evaluated" },
  { name: "sdk/original/helper.d.cts", data: "export {};" },
  { name: "sdk/original/helper.d.mts", data: "export {};" },
  { name: "records.jsonl", data: "{}\n" },
];
test("whole archive custody verifies every CRC, including the real reached SDK declaration spellings", () => {
  const f = archiveFixture(safe),
    members = decodeCapturedArchive(f.archive, f.entry);
  assert.equal(members.size, 4);
  assert.equal(members.get("sdk/original/helper.d.cts")!.toString(), "export {};");
});
test("whole archive refuses corrupt, duplicate, traversing, binary and unexpected members without native or product execution", () => {
  for (const files of [
    [...safe, { name: "../escape.json", data: "x" }],
    [...safe, safe[0]!],
    [...safe, { name: "native.node", data: "inert" }],
    [...safe, { name: "sdk/original/unknown.js", data: "throw Error('never evaluated')" }],
  ]) {
    const f = archiveFixture(files);
    assert.throws(() => decodeCapturedArchive(f.archive, f.entry));
  }
  const f = archiveFixture(safe),
    corrupt = Buffer.from(f.archive),
    first = zipEntries(corrupt)[0]!;
  const start =
    first.localHeaderOffset +
    30 +
    corrupt.readUInt16LE(first.localHeaderOffset + 26) +
    corrupt.readUInt16LE(first.localHeaderOffset + 28);
  corrupt[start] = corrupt[start]! ^ 1;
  const adjusted = {
    ...f.entry,
    reference: { ...f.entry.reference, archiveDigest: "sha256:" + sha256(corrupt) },
  };
  assert.throws(() => decodeCapturedArchive(corrupt, adjusted), /CRC/u);
  for (const change of [
    (e: CapturePolicyEntry) => {
      e.artifact.decodedBytes++;
    },
    (e: CapturePolicyEntry) => {
      e.reference.captureMemberSha256 = "0".repeat(64);
    },
  ]) {
    const e = structuredClone(f.entry);
    change(e);
    assert.throws(() => decodeCapturedArchive(f.archive, e));
  }
});
function strictTypeBoundary(pending: UnqualifiedCapture): void {
  // @ts-expect-error Pending installs do not satisfy Bundle's strict InstallReceipt.
  const strict: Bundle = pending;
  void strict;
}
void strictTypeBoundary;
