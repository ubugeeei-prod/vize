import assert from "node:assert/strict";
import test from "node:test";
import { mkdtemp, mkdir, readFile, rm, symlink, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import {
  colorCaptureLaw,
  otherColorPNGs,
} from "../../../tools/support/release/public_acceptance/vrt_trusted_color_capture.ts";
import type { VrtReport } from "../../../tools/support/release/public_acceptance/vrt_artifacts.ts";
import { changedColorSource } from "../../../tools/support/release/public_acceptance/vrt_trusted_color_build.ts";

/** Pure input/model controls only. No native builder, Browser, service or tunnel launch. */
function colorPacket(
  side: "left" | "right",
  phase: "new" | "match" | "diff" | "update" | "updated-match",
) {
  const failed = side === "left" && (phase === "diff" || phase === "update");
  return {
    success: true,
    summary: {
      total: 1,
      new: phase === "new" ? 1 : 0,
      passed: phase !== "new" && !failed ? 1 : 0,
      failed: failed ? 1 : 0,
      errors: 0,
      skipped: 0,
      duration: 0,
    },
    results: [
      {
        artPath: `/owned/src/${side}/Button.art.vue`,
        variantName: "Default",
        viewport: "authored-compact",
        passed: !failed,
        snapshotPath: "/owned/snapshot.png",
        currentPath: "/owned/current.png",
        ...(phase === "new" ? { isNew: true } : { diffPercentage: failed ? 1 : 0 }),
        images: {
          snapshot: "/artifacts/snapshot",
          current: "/artifacts/current",
          ...(failed ? { diff: "/artifacts/diff" } : {}),
        },
      },
    ],
    artifacts: {
      snapshotDir: "/owned",
      jsonReportPath: "/owned/report.json",
      htmlReportPath: "/owned/report.html",
    },
    reports: { json: "/artifacts/json", html: "/artifacts/html" },
  };
}
void test("same-basename model binds every new/match/diff/update/match phase separately", () => {
  for (const side of ["left", "right"] as const)
    for (const phase of ["new", "match", "diff", "update", "updated-match"] as const)
      colorCaptureLaw(colorPacket(side, phase), `/owned/src/${side}/Button.art.vue`, side, phase);
  for (const mutate of [
    (data: ReturnType<typeof colorPacket>) => {
      data.summary.failed = 0;
    },
    (data: ReturnType<typeof colorPacket>) => {
      Reflect.set(data.results[0], "diffPercentage", 0);
    },
    (data: ReturnType<typeof colorPacket>) => {
      data.results[0].artPath = "/owned/src/right/Button.art.vue";
    },
    (data: ReturnType<typeof colorPacket>) => {
      Reflect.deleteProperty(data.results[0].images, "diff");
    },
  ]) {
    const data = colorPacket("left", "diff");
    mutate(data);
    assert.throws(() => colorCaptureLaw(data, "/owned/src/left/Button.art.vue", "left", "diff"));
  }
});
void test("authored color change is exactly one replacement in the frozen whole source", async () => {
  const blue = await readFile(
    new URL("../fixtures/musea/snapshot-collision/left/Button.art.vue", import.meta.url),
  );
  assert.deepEqual(changedColorSource(blue, false), blue);
  assert.equal(
    changedColorSource(blue, true).toString(),
    blue.toString().replace("#0000ff", "#00ff00"),
  );
  assert.throws(() => changedColorSource(Buffer.from("#0000ff"), true));
});

void test("real other-Art files: every snapshot/current/diff byte mutation and ownership escape refuses", async (t) => {
  const temporary = await mkdtemp(path.join(os.tmpdir(), "vrt-other-color-files-"));
  t.after(() => rm(temporary, { recursive: true, force: true }));
  const root = path.join(temporary, "owned"),
    art = path.join(root, "src/left/Button.art.vue");
  await mkdir(root);
  const fields = ["snapshotPath", "currentPath", "diffPath"] as const;
  const files = Object.fromEntries(fields.map((field) => [field, path.join(root, `${field}.png`)]));
  for (const field of fields) await writeFile(files[field], `whole physical ${field} bytes\n`);
  const report: VrtReport = {
    reportOwner: { version: 1, artIdentity: "src/left/Button.art.vue" },
    summary: colorPacket("left", "diff").summary,
    results: [
      {
        artPath: art,
        art: "Button",
        variant: "Default",
        variantName: "Default",
        viewport: "authored-compact",
        status: "failed",
        snapshotPath: files.snapshotPath,
        currentPath: files.currentPath,
        diffPath: files.diffPath,
      },
    ],
  };
  const before = await otherColorPNGs(root, art, report);
  assert.deepEqual(
    before.map((image) => image.field),
    fields,
  );
  for (const image of before) {
    await writeFile(image.file, Buffer.concat([image.bytes, Buffer.from("changed")]));
    await assert.rejects(async () =>
      assert.deepEqual(await otherColorPNGs(root, art, report), before),
    );
    await writeFile(image.file, image.bytes);
  }
  assert.deepEqual(await otherColorPNGs(root, art, report), before);
  const outside = path.join(temporary, "outside.png");
  await writeFile(outside, "outside bytes\n");
  const result = report.results[0];
  result.currentPath = outside;
  await assert.rejects(otherColorPNGs(root, art, report));
  const link = path.join(root, "linked.png");
  await symlink(outside, link);
  result.currentPath = link;
  await assert.rejects(otherColorPNGs(root, art, report));
  const directory = path.join(root, "linked-directory");
  await symlink(temporary, directory);
  result.currentPath = path.join(directory, "outside.png");
  await assert.rejects(otherColorPNGs(root, art, report));
  result.currentPath = files.currentPath;
  report.reportOwner!.artIdentity = "src/right/Button.art.vue";
  await assert.rejects(otherColorPNGs(root, art, report));
  report.reportOwner!.artIdentity = "src/left/Button.art.vue";
  result.artPath = path.join(root, "src/right/Button.art.vue");
  await assert.rejects(otherColorPNGs(root, art, report));
});
