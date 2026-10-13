import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { ensureHostedLiteralSource } from "./warm-type-backed-git-reader-literal-source.ts";
import {
  admittedReview,
  bodyCell,
  type Cell,
  type qualifyDualCurrentFrame,
} from "./warm-type-backed-git-reader-dual-frame.ts";

type Packet = ReturnType<typeof qualifyDualCurrentFrame>;
type OriginalReceipt = { completeCurrentPairs: number };
type Refusals = {
  completeCurrentPairs: number;
  squashedEquivalentPairs: number;
  squashedParent: string;
  squashedCommit: string;
  controls: Array<{ name: string; message: string }>;
};
type Retained = {
  source: string;
  completeRetainedBodies: number;
  passed: number;
  skipped: number;
  stdout: string;
  stderr: string;
};
type Factory = {
  qualifyOwnedReaderFrame(root: string): OriginalReceipt;
  qualifyComponentReaderFrame(root: string): OriginalReceipt;
  exerciseOwnedReaderFrameRefusals(root: string): Refusals;
  exerciseComponentReaderFrameRefusals(root: string): Refusals;
  runRetainedReaderLaw(root: string): Retained;
};
const git = (root: string, args: string[], input?: Buffer | string) =>
  execFileSync("git", ["--no-replace-objects", ...args], {
    cwd: root,
    input,
    maxBuffer: 64 * 1024 * 1024,
    env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
  });
function sourceBody(root: string, cell: Cell): Buffer {
  const body = git(root, ["cat-file", "blob", cell.blob]);
  assert.deepEqual(bodyCell(body), cell);
  return body;
}
function provisionOriginalObjects(root: string, source: string, directory: string, module: string) {
  const file = path.join(import.meta.dirname, directory, module.replace(/\.ts$/u, ".json"));
  const metadata = JSON.parse(fs.readFileSync(file, "utf8"));
  const literals = new Set<string>([
    source,
    metadata.originalSource,
    metadata.retainedSource,
    "3dda8f9d71fbd7e701f0805187daff6191865190",
    ...metadata.rawIntegrationPath.map((node: { source: string }) => node.source),
  ]);
  for (const source of literals) ensureHostedLiteralSource(root, source);
  source = metadata.retainedSource;
  for (let count = 0; count < 64; count++) {
    ensureHostedLiteralSource(root, source);
    const body = git(root, ["cat-file", "commit", source]);
    assert.equal(
      createHash("sha1").update(`commit ${body.length}\0`).update(body).digest("hex"),
      source,
    );
    if (source === metadata.originalSource) return;
    const parent = /^parent ([a-f0-9]{40})$/mu.exec(
      body.subarray(0, body.indexOf("\n\n")).toString(),
    );
    assert(parent, "authenticated bounded historical first parent required");
    source = parent[1];
  }
  assert.fail("historical first-parent provisioning exceeds unchanged64 limit");
}
async function withOriginal<T>(
  root: string,
  packet: Packet,
  role: "sdk" | "component",
  operation: (view: string, factory: Factory) => T,
) {
  const review = admittedReview(root, packet);
  const original = review.original[role];
  provisionOriginalObjects(root, original.source, original.directory, original.module);
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "vize-dual-original-source-"));
  try {
    git(fixture, ["init", "--quiet"]);
    const common = git(root, ["rev-parse", "--git-common-dir"]).toString().trim();
    fs.writeFileSync(
      path.join(fixture, ".git/objects/info/alternates"),
      path.resolve(root, common, "objects") + "\n",
    );
    for (const row of original.rows) {
      const body = sourceBody(root, row.currentAfter);
      const target = path.join(fixture, row.file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, body, { flag: "wx", mode: 0o644 });
    }
    git(fixture, ["add", "--", "."]);
    const tree = git(fixture, ["write-tree"]).toString().trim();
    const commit = git(fixture, [
      "-c",
      "user.name=SourceFixture",
      "-c",
      "user.email=fixture@example.invalid",
      "commit-tree",
      tree,
      "-m",
      "Authenticated original111 source-control view",
    ])
      .toString()
      .trim();
    git(fixture, ["update-ref", "refs/heads/original-frame", commit]);
    git(fixture, ["symbolic-ref", "HEAD", "refs/heads/original-frame"]);
    const module = path.join(import.meta.dirname, original.directory, original.module);
    const factory = await import(pathToFileURL(module).href);
    return await operation(fixture, factory);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
}
export async function exerciseDualOriginalFrameControls(root: string, packet: Packet) {
  const owned = await withOriginal(root, packet, "sdk", (view, module) => {
    const receipt = module.qualifyOwnedReaderFrame(view);
    assert.equal(receipt.completeCurrentPairs, 111);
    return module.exerciseOwnedReaderFrameRefusals(view);
  });
  const component = await withOriginal(root, packet, "component", (view, module) => {
    const receipt = module.qualifyComponentReaderFrame(view);
    assert.equal(receipt.completeCurrentPairs, 111);
    return module.exerciseComponentReaderFrameRefusals(view);
  });
  return {
    owned,
    component,
    scope: "Both unchanged original helper/refusal sets; actual current112 gated first",
    runtimeCredit: false,
  };
}
export async function runDualRetainedReaderLaw(root: string, packet: Packet) {
  const review = admittedReview(root, packet);
  assert.deepEqual(review.original.sdk.retained, review.original.component.retained);
  return withOriginal(root, packet, "sdk", (view, module) => module.runRetainedReaderLaw(view));
}
