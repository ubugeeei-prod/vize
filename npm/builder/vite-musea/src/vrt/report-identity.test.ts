import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, rm, symlink, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {
  assertArtReportOwnership,
  attachArtReportOwner,
  resolveArtReportTarget,
} from "./report-identity.ts";

void test("safe ordinary filenames survive while collisions use portable, order-independent owners", () => {
  const root = path.resolve("/project-a");
  const arts = ["src/left/Button.art.vue", "src/right/Button.art.vue"];
  const resolve = (art: string, all: string[], project = root) =>
    resolveArtReportTarget(art, all, project, path.join(project, "reports"));
  assert.equal(path.basename(resolve(arts[0], [arts[0]]).jsonReportPath), "vrt-Button-report.json");
  const left = resolve(arts[0], arts);
  const right = resolve(arts[1], arts);
  assert.equal(left.identity, "src/left/Button.art.vue");
  assert.match(path.basename(left.jsonReportPath), /^vrt-art-[0-9a-f]{64}-report\.json$/);
  assert.notEqual(left.jsonReportPath, right.jsonReportPath);
  assert.deepEqual(resolve(arts[0], arts.toReversed()), left);
  const moved = resolve(arts[0], arts, path.resolve("/project-b"));
  assert.equal(path.basename(moved.jsonReportPath), path.basename(left.jsonReportPath));
  const folded = resolve("src/button.art.vue", ["src/Button.art.vue", "src/button.art.vue"]);
  assert.match(path.basename(folded.jsonReportPath), /^vrt-art-[0-9a-f]{64}-report\.json$/);
});

void test("unsafe or reserved names cannot impersonate qualified report owners", () => {
  const root = path.resolve("/project");
  for (const basename of ["Probe: name", "日本語", `art-${"a".repeat(64)}`, "A".repeat(201)]) {
    const art = `src/${basename}.art.vue`;
    const target = resolveArtReportTarget(art, [art], root, root);
    assert.match(path.basename(target.htmlReportPath), /^vrt-art-[0-9a-f]{64}-report\.html$/);
    assert.notEqual(path.basename(target.htmlReportPath), `vrt-${basename}-report.html`);
  }
  assert.throws(
    () => resolveArtReportTarget("src/Missing.art.vue", ["src/Actual.art.vue"], root, root),
    /not found/,
  );
  assert.throws(
    () => resolveArtReportTarget("../Outside.art.vue", ["../Outside.art.vue"], root, root),
    /outside/,
  );
  assert.throws(
    () =>
      resolveArtReportTarget(
        "src/Button.art.vue",
        ["src/Button.art.vue", "src/Button.art.vue"],
        root,
        root,
      ),
    /Ambiguous/,
  );
});

void test("existing reports require proved ownership and refusals preserve their complete bytes", async (t) => {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-report-owner-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  const reportDir = path.join(root, "reports");
  const art = path.join(root, "src/left/Button.art.vue");
  const foreign = path.join(root, "src/right/Button.art.vue");
  const target = resolveArtReportTarget(art, [art], root, reportDir);
  await assertArtReportOwnership(target, root);
  await mkdir(reportDir);
  const html = Buffer.from("<!doctype html><title>original owner</title>\n");
  await writeFile(target.htmlReportPath, html);
  const legacy = JSON.stringify({ results: [{ artPath: art }] });
  await writeFile(target.jsonReportPath, legacy);
  await assertArtReportOwnership(target, root);
  assert.equal(await readFile(target.jsonReportPath, "utf8"), legacy);
  assert.deepEqual(await readFile(target.htmlReportPath), html);
  const portable = attachArtReportOwner(
    JSON.stringify({ results: [{ artPath: "C:\\old-host\\Button.art.vue" }] }),
    target,
  );
  await writeFile(target.jsonReportPath, portable);
  await assertArtReportOwnership(target, root);
  for (const bytes of [
    "{broken",
    JSON.stringify({ results: [] }),
    JSON.stringify({ results: [{ artPath: foreign }] }),
    JSON.stringify({ results: [{ artPath: art }, { artPath: foreign }] }),
    JSON.stringify({ reportOwner: { version: 2, artIdentity: target.identity } }),
    JSON.stringify({ reportOwner: { version: 1, artIdentity: "src/right/Button.art.vue" } }),
  ]) {
    await writeFile(target.jsonReportPath, bytes);
    await assert.rejects(assertArtReportOwnership(target, root), /Archive or move/);
    assert.equal(await readFile(target.jsonReportPath, "utf8"), bytes);
    assert.deepEqual(await readFile(target.htmlReportPath), html);
  }
  await rm(target.jsonReportPath);
  await assert.rejects(assertArtReportOwnership(target, root), /Archive or move/);
  assert.deepEqual(await readFile(target.htmlReportPath), html);
  await writeFile(path.join(root, "outside.json"), legacy);
  await symlink(path.join(root, "outside.json"), target.jsonReportPath);
  await assert.rejects(assertArtReportOwnership(target, root), /not a regular file/);
  assert.equal(await readFile(path.join(root, "outside.json"), "utf8"), legacy);
});

void test("case-fold aliases from a removed Art are refused even on case-sensitive hosts", async (t) => {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-report-fold-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  const art = "src/Button.art.vue";
  const target = resolveArtReportTarget(art, [art], root, root);
  const existing = path.join(root, "vrt-button-report.json");
  const bytes = JSON.stringify({ results: [{ artPath: "src/button.art.vue" }] });
  await writeFile(existing, bytes);
  await assert.rejects(assertArtReportOwnership(target, root), /Archive or move/);
  assert.equal(await readFile(existing, "utf8"), bytes);
});
