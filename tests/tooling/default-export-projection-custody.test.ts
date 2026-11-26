import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import nodePath from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { beforeDefaultExportProjection } from "../../tools/support/compat/content-mapper/default-export-projection-transition.ts";
import { beforeGenericHoistedAlias } from "../../tools/support/compat/content-mapper/generic-hoisted-alias-transition.ts";
import { beforeSharedTemplateRefTargetsSource } from "../../tools/support/compat/content-mapper/shared-template-ref-targets-transition.ts";
import { beforeCheckModeGenerator } from "../../tools/support/compat/content-mapper/check-mode-successor.ts";
import { beforeComponentHoverGaps } from "../../tools/support/compat/content-mapper/component-hover-gaps-transition.ts";
import { beforeLibTypeShadowingAudit } from "../../tools/support/compat/content-mapper/lib-type-shadowing-audit-transition.ts";

const root = new URL("../../", import.meta.url);
const read = (path: string) => readFileSync(new URL(path, root), "utf8");
const identity = (text: string) => ({
  bytes: Buffer.byteLength(text),
  sha256: createHash("sha256").update(text).digest("hex"),
});
const path = "crates/vize_canon/src/virtual_ts/generator.rs";
const raw = read("tests/_fixtures/tooling/default-export-projection-13693.json");
assert.equal(
  identity(raw).sha256,
  "24d58a6b250d5ac39cde3db348235cff88c6517ae4d155b1b7fefda92888098f",
);
const record = JSON.parse(raw);
const [file] = record.files;

void test("default export projection restores the exact pinned generator and round trips all bytes", () => {
  assert.deepEqual(record.credit, { compiled: false, native: false, performance: false });
  assert.deepEqual(record.predecessor, {
    revision: "6bf40dbded97b320d607a446d036d77829cf761d",
    gitBlob: "ef50d92d7dd5d920d4b72d6924ee7dca0a6f5119",
  });
  assert.deepEqual(
    record.files.map((entry: { path: string }) => entry.path),
    [path],
  );
  const predecessor = JSON.parse(read("tests/_fixtures/tooling/script-consumed-slots-13578.json"));
  assert.deepEqual(
    file.before,
    predecessor.files.find((entry: { path: string }) => entry.path === path).after,
  );
  const current = read(path);
  assert.deepEqual(identity(current), file.after);
  const previous = beforeDefaultExportProjection(path, current);
  assert.deepEqual(identity(previous), file.before);
  let roundTrip = previous;
  for (const hunk of file.hunks) {
    assert.equal(roundTrip.split(hunk.before).length - 1, 1);
    roundTrip = roundTrip.replace(hunk.before, () => hunk.after);
  }
  assert.equal(roundTrip, current);
  assert.equal(beforeDefaultExportProjection(path, previous), previous);
  assert.equal(beforeDefaultExportProjection("other.rs", current), current);
  for (const changed of [
    current + current,
    current + "\n",
    ...file.hunks.map((hunk: { after: string }) => current.replace(hunk.after, "")),
  ]) {
    assert.equal(beforeDefaultExportProjection(path, changed), changed);
    assert.notDeepEqual(identity(changed), file.before);
  }
});

void test("the actual verifier rejects every changed, missing, and moved supporting source", (t) => {
  assert.deepEqual(
    record.supportingSources.map((source: { path: string }) => source.path),
    [
      "crates/vize_canon/src/virtual_ts/generator/irregular_default.rs",
      "crates/vize_canon/src/virtual_ts/generator/module_projection.rs",
      "crates/vize_canon/src/virtual_ts/generator/options_api.rs",
      "crates/vize_canon/src/virtual_ts/generator/options_api/default_export.rs",
      "crates/vize_canon/src/virtual_ts/generator/options_api_props_identifiers.rs",
      "crates/vize_canon/src/virtual_ts/generator/options_api_support.rs",
      "crates/vize_canon/src/virtual_ts/generator/spans.rs",
      "crates/vize_canon/src/virtual_ts/generator/standard_default.rs",
      "crates/vize_canon/src/virtual_ts/generator/template_ref_keys.rs",
      "crates/vize_canon/src/virtual_ts/generator/template_ref_keys/lines.rs",
      "crates/vize_canon/src/virtual_ts/generator/template_ref_keys/lines/tsx.rs",
    ],
  );
  const directory = mkdtempSync(nodePath.join(os.tmpdir(), "vize-default-export-custody-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const sourceRoot = pathToFileURL(directory + nodePath.sep);
  for (const source of record.supportingSources) {
    const target = new URL(source.path, sourceRoot);
    mkdirSync(new URL(".", target), { recursive: true });
    writeFileSync(target, readFileSync(new URL(source.path, root)));
  }
  const current = read(path);
  const expected = beforeDefaultExportProjection(path, current);
  assert.equal(beforeDefaultExportProjection(path, current, sourceRoot), expected);
  for (const source of record.supportingSources) {
    const target = new URL(source.path, sourceRoot);
    const bytes = readFileSync(target);
    const text = bytes.toString("utf8");
    assert.equal(text.split(source.callee).length - 1, 1);
    const offset = bytes.indexOf(source.callee) + 3;
    const changed = Buffer.from(bytes);
    changed.writeUInt8(changed.readUInt8(offset) ^ 1, offset);
    writeFileSync(target, changed);
    assert.throws(() => beforeDefaultExportProjection(path, current, sourceRoot));
    rmSync(target);
    assert.throws(() => beforeDefaultExportProjection(path, current, sourceRoot));
    const moved = new URL(source.path + ".moved", sourceRoot);
    writeFileSync(moved, bytes);
    assert.throws(() => beforeDefaultExportProjection(path, current, sourceRoot));
    renameSync(moved, target);
    assert.equal(beforeDefaultExportProjection(path, current, sourceRoot), expected);
  }
});

void test("the new generator edge preserves every older authenticated source transition", () => {
  const current = read(path);
  for (const [name, before] of [
    ["generic-hoisted-alias-forwarding", beforeGenericHoistedAlias],
    ["shared-template-ref-targets", beforeSharedTemplateRefTargetsSource],
    ["component-hover-gaps-12461", beforeComponentHoverGaps],
    ["lib-type-shadowing-audit-12536", beforeLibTypeShadowingAudit],
  ] as const) {
    const legacy = JSON.parse(read(`tests/_fixtures/tooling/${name}.json`));
    const producer = legacy.files.find((entry: { path: string }) => entry.path === path);
    assert.deepEqual(identity(before(path, current)), producer.before, name);
  }
  const beforeCheckMode = beforeCheckModeGenerator(
    path,
    beforeSharedTemplateRefTargetsSource(path, current),
  );
  assert.deepEqual(identity(beforeCheckMode), {
    bytes: 37699,
    sha256: "31ffef289d05da9ba517f41367ff6d21e46ef76e40b09a26a4389ceb7bdd6655",
  });
});
