import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { digest } from "./fixtures.ts";

const variants = {
  Controls: ["Default", "Custom theme"],
  Keys: ["__proto__", "constructor", "hasOwnProperty"],
};
interface Preview {
  sectionName: string;
  sectionId: string;
  ariaControls: string;
  html: string;
}
interface Result {
  artPath: string;
  viewport: string;
  variantName: string;
  snapshotPath: string;
  currentPath: string;
  passed: boolean;
  isNew?: boolean;
  diffPercentage: number;
}
interface DevRow {
  title: keyof typeof variants;
  phase: string;
  rendered: Preview[];
  hashes: string[];
  groups: string[];
  renderFailure?: string;
  data: {
    summary: {
      total: number;
      new: number;
      passed: number;
      failed: number;
      skipped: number;
      errors: number;
    };
    results: Result[];
  };
}

/** Consume all original case receipts and the actual retained response/report/PNG bytes. */
export function assertObservations(artifactRoot: string) {
  const read = (file: string) => JSON.parse(fs.readFileSync(path.join(artifactRoot, file), "utf8"));
  const staticReceipt = read("static/observations.json"),
    dev = read("dev/observations.json");
  for (const receipt of [staticReceipt, dev]) {
    assert.deepEqual(receipt.errors, []);
    assert.deepEqual(receipt.consoleErrors, []);
    assert.equal(receipt.sources.length, 2);
    for (const source of receipt.sources)
      assert.equal(digest(Buffer.from(source.bytes)), source.sha256);
  }
  const rows = staticReceipt.observations;
  assert.equal(rows.length, 9, "complete original static receipt denominator");
  assert.deepEqual(
    rows.slice(0, 5).map((row: { title: string; name: string }) => [row.title, row.name]),
    Object.entries(variants).flatMap(([title, names]) => names.map((name) => [title, name])),
  );
  for (const row of rows.slice(0, 5)) {
    assert.match(row.html, /^<!doctype html>|^<html\b/iu);
    assert.match(row.physicalSha256, /^[a-f0-9]{64}$/u);
    assert.match(new URL(row.url).pathname, /^\/__musea__\/preview\/[a-f0-9]{20}\.html$/u);
  }
  assert.equal(new Set(rows.slice(0, 5).map((row: { url: string }) => row.url)).size, 5);
  for (const [index, title] of ["Controls", "Keys"].entries()) {
    const row = rows[index + 5];
    assert.equal(row.phase, `${title.toLowerCase()}-gallery`);
    assert.deepEqual(
      row.previews.map((frame: Preview) => frame.sectionName),
      variants[title as keyof typeof variants],
    );
    for (const frame of row.previews as Preview[]) assert.match(frame.html, /^<html\b/u);
  }
  assert.equal(rows[7].manifest.length, 2);
  assert.equal(rows[7].controlGlobals.length, 2);
  assert.deepEqual(rows[8], {
    phase: "literal-nav-selected",
    name: "__proto__",
    sectionId: "variant-proto",
    ariaControls: "variant-proto",
  });
  assert.deepEqual(
    dev.observations.map((row: DevRow) => [row.title, row.phase]),
    [
      ["Controls", "initial"],
      ["Controls", "repeat"],
      ["Keys", "initial"],
      ["Keys", "repeat"],
    ],
  );
  const pngReceipts: Array<{
      title: string;
      phase: string;
      name: string;
      snapshotPath: string;
      bytes: number;
      sha256: string;
    }> = [],
    initialPaths: string[] = [],
    initialHashes: string[] = [];
  for (const row of dev.observations as DevRow[]) {
    const names = variants[row.title],
      initial = row.phase === "initial";
    assert.equal(row.renderFailure, undefined);
    assert.deepEqual(row.groups, names);
    assert.deepEqual(
      row.rendered.map((frame) => frame.sectionName),
      names,
    );
    for (const frame of row.rendered) assert.match(frame.html, /^<html\b/u);
    for (const [key, count] of Object.entries({
      total: names.length,
      new: initial ? names.length : 0,
      passed: initial ? 0 : names.length,
      failed: 0,
      skipped: 0,
      errors: 0,
    }))
      assert.equal(row.data.summary[key as keyof DevRow["data"]["summary"]], count);
    assert.deepEqual(
      row.data.results.map((result) => result.variantName),
      names,
    );
    assert.equal(new Set(row.data.results.map((result) => result.snapshotPath)).size, names.length);
    const directory = `dev/${row.title}-${row.phase}`;
    const response = read(`${directory}/response.json`),
      report = read(`${directory}/report.json`);
    assert.deepEqual(response.summary, row.data.summary);
    assert.deepEqual(response.results, row.data.results);
    assert.deepEqual(report.summary, row.data.summary);
    const fields = [
      "artPath",
      "variantName",
      "viewport",
      "snapshotPath",
      "currentPath",
      "diffPercentage",
    ];
    const identity = (item: Record<string, unknown>) =>
      Object.fromEntries(fields.map((key) => [key, item[key]]));
    assert.deepEqual(
      report.results.map(identity),
      row.data.results.map((item) => identity({ ...item })),
    );
    for (const result of report.results) {
      assert.equal(result.art, row.title);
      assert.equal(result.variant, result.variantName);
      assert.equal(result.status, initial ? "new" : "passed");
    }
    assert.ok(
      row.data.results.every((result) => result.passed && Boolean(result.isNew) === initial),
    );
    assert.match(
      fs.readFileSync(path.join(artifactRoot, directory, "report.html"), "utf8"),
      /<html\b/u,
    );
    assert.match(
      fs.readFileSync(path.join(artifactRoot, `dev/${row.title}-${row.phase}.html`), "utf8"),
      /<html\b/u,
    );
    const hashes = row.data.results.map((result, index) => {
      const bytes = fs.readFileSync(path.join(artifactRoot, directory, `${index}.png`));
      assert.equal(bytes.subarray(0, 8).toString("hex"), "89504e470d0a1a0a");
      const sha256 = digest(bytes);
      pngReceipts.push({
        title: row.title,
        phase: row.phase,
        name: result.variantName,
        snapshotPath: result.snapshotPath,
        bytes: bytes.length,
        sha256,
      });
      return sha256;
    });
    assert.deepEqual(hashes, row.hashes);
    assert.equal(new Set(hashes).size, names.length);
    if (initial) {
      initialPaths.push(...row.data.results.map((result) => result.snapshotPath));
      initialHashes.push(...hashes);
    } else {
      const before = dev.observations.find(
        (item: DevRow) => item.title === row.title && item.phase === "initial",
      );
      assert.deepEqual(hashes, before.hashes);
      assert.deepEqual(
        row.data.results.map((result) => result.snapshotPath),
        before.data.results.map((result: Result) => result.snapshotPath),
      );
      assert.ok(row.data.results.every((result) => result.diffPercentage === 0));
    }
  }
  assert.equal(new Set(initialPaths).size, 5);
  assert.equal(new Set(initialHashes).size, 5);
  assert.equal(pngReceipts.length, 10);
  return { static: staticReceipt, dev, pngReceipts };
}
