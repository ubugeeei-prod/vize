import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import path from "node:path";
import type { Page } from "playwright";
import { ownershipIndex, type Evidence } from "./vrt_artifacts.ts";
import { sha256, sides, type Side } from "./vrt_fixtures.ts";
import { rebuildTrustedColors, type TrustedColorBuild } from "./vrt_trusted_color_build.ts";
import { capturePublicColor } from "./vrt_trusted_color_capture.ts";
import type { PublicBodyCustody } from "./vrt_trusted_gallery.ts";
import type { PublicSession } from "./vrt_trusted_ui.ts";

/** HTTPS six states on the SAME published consumer/gallery/loopback session and physical owners. */
export async function observePublicColors(input: {
  page: Page;
  session: PublicSession;
  build: TrustedColorBuild;
  gallery: string;
  bodies: PublicBodyCustody;
  e: Evidence;
  records: unknown[];
  setPhase: (phase: string) => void;
  assertRunning: () => void;
}) {
  const { page, session, build, gallery, bodies, e, records, setPhase, assertRunning } = input;
  const started = Date.now(),
    first = {} as Record<Side, Buffer>,
    updated = {} as Record<Side, Buffer>;
  const snapshots = {} as Record<Side, string>,
    reports = {} as Record<Side, string>;
  let index: Awaited<ReturnType<typeof ownershipIndex>> | undefined,
    captures = 0;
  for (const phase of ["new", "match", "diff", "update", "updated-match"] as const) {
    if (phase === "diff") {
      setPhase("HTTPS-physical-authored-green-rebuild");
      await bodies.settle();
      const rightBefore = await readFile(snapshots.right),
        leftBefore = await readFile(snapshots.left);
      await rebuildTrustedColors(build, e, "public-authored-green", true);
      await assert.rejects(stat(path.join(build.literal.root, "src")), { code: "ENOENT" });
      assert.deepEqual(await readFile(snapshots.right), rightBefore);
      assert.deepEqual(await readFile(snapshots.left), leftBefore);
      assert.deepEqual(await bodies.fetch(new URL("api/static.json", gallery)), build.literal.raw);
      await page.goto(gallery);
      await bodies.settle();
      records.push({
        phase: "HTTPS-physical-authored-green-rebuild",
        manifestSha256: sha256(build.literal.raw),
        originalBaselinePNGsUnchanged: true,
        authoredSourcesDeleted: true,
      });
      assertRunning();
    }
    const phaseResults: Array<Awaited<ReturnType<typeof capturePublicColor>>> = [];
    for (const side of sides) {
      setPhase(`HTTPS-${side}-${phase}`);
      assertRunning();
      const result = await capturePublicColor({
        page,
        session,
        build,
        gallery,
        bodies,
        side,
        phase,
        e,
        first: first[side],
        updated: updated[side],
      });
      captures++;
      if (phase === "new") {
        first[side] = result.baseline;
        snapshots[side] = result.receipt.snapshotPath;
        reports[side] = result.receipt.reports[0].file;
      } else {
        assert.equal(result.receipt.snapshotPath, snapshots[side]);
        assert.equal(result.receipt.reports[0].file, reports[side]);
      }
      if (phase === "update") updated[side] = result.baseline;
      if (phase === "diff") assert.deepEqual(await readFile(snapshots[side]), first[side]);
      if (phase !== "new") {
        assert.ok(index);
        assert.deepEqual(
          (await ownershipIndex(path.join(build.literal.root, "trusted-baselines"))).bytes,
          index.bytes,
        );
      }
      records.push({ ...result.receipt, phase: `HTTPS-${side}-${phase}` });
      phaseResults.push(result);
      assertRunning();
    }
    if (phase === "new") {
      assert.notEqual(snapshots.left, snapshots.right);
      assert.notEqual(reports.left, reports.right);
      assert.notDeepEqual(first.left, first.right);
      index = await ownershipIndex(path.join(build.literal.root, "trusted-baselines"));
      await e.save("trusted-public/colors/index-before-rebuild.json", index.bytes);
    }
    const summaries = phaseResults.map((result) => result.receipt.data.summary);
    const total = (key: "total" | "new" | "passed" | "failed" | "errors" | "skipped") =>
      summaries.reduce((sum, summary) => sum + summary[key], 0);
    assert.deepEqual(
      [
        total("total"),
        total("new"),
        total("passed"),
        total("failed"),
        total("errors"),
        total("skipped"),
      ],
      phase === "new"
        ? [2, 2, 0, 0, 0, 0]
        : phase === "diff" || phase === "update"
          ? [2, 0, 1, 1, 0, 0]
          : [2, 0, 2, 0, 0, 0],
    );
    records.push({ phase: `HTTPS-two-Art-${phase}`, separateRealRequests: 2, summaries });
  }
  assert.equal(captures, 10);
  assert.deepEqual(await readFile(snapshots.left), updated.left);
  assert.notDeepEqual(updated.left, first.left);
  assert.deepEqual(updated.right, first.right);
  assert.deepEqual(await readFile(snapshots.right), first.right);
  assert.ok(index);
  const restored = await ownershipIndex(path.join(build.literal.root, "trusted-baselines"));
  assert.deepEqual(restored.bytes, index.bytes);
  await e.save("trusted-public/colors/index-after-updated-match.json", restored.bytes);
  records.push({
    phase: "HTTPS-six-state-authored-campaign",
    states: ["new", "match", "physical-rebuild", "diff", "update", "match"],
    captures,
    actualElapsedMs: Date.now() - started,
    sameSessionAndConsumer: true,
    rightWholeBaselineUnchanged: true,
    snapshotAndReportOwnerPathsRestoredExactly: true,
  });
}
