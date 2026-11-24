// The opt-in implementation must not change stock code; shared production fixes may.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { verifyNativeAddonImages } from "../../../commands/levels/native-addon-identity.ts";

interface Change {
  status: string;
  path: string;
}

const gated = new Set([
  "crates/vize_atelier_sfc/src/bundler/blocks/products.rs",
  "crates/vize_vitrine/src/napi/sfc/bundler/products.rs",
]);
const tests = new Set([
  "crates/vize_atelier_sfc/tests/bundler_block_products.rs",
  "tests/tooling/native-addon-identity.test.ts",
  "tests/tooling/stock-addon-qualification.test.ts",
]);

export function classifyStockAddonChanges(changes: Change[]) {
  const production = changes.filter(
    (change) =>
      !["A", "M"].includes(change.status) ||
      !(
        gated.has(change.path) ||
        tests.has(change.path) ||
        (change.path.startsWith("docs/") && change.path.endsWith(".md"))
      ),
  );
  return {
    mode: production.length ? "stock-runtime" : "byte-identity",
    reason: production.length
      ? "shared, unknown, or removed source requires stock runtime qualification"
      : "only feature-gated products and nonshipping tests or documentation changed",
    production,
  };
}

export function qualifyStockAddon(root: string, base: string, head: string) {
  for (const revision of [base, head]) assert.match(revision, /^[a-f0-9]{40}$/u);
  const fields = execFileSync(
    "git",
    ["diff", "--name-status", "--no-renames", "-z", base, head, "--"],
    { cwd: root, encoding: "utf8" },
  ).split("\0");
  assert.equal(fields.pop(), "");
  assert.equal(fields.length % 2, 0);
  const changes: Change[] = [];
  for (let i = 0; i < fields.length; i += 2) {
    assert.match(fields[i], /^[AMDTUXB]$/u);
    assert.ok(fields[i + 1]);
    changes.push({ status: fields[i], path: fields[i + 1] });
  }
  return { base, head, changes, ...classifyStockAddonChanges(changes) };
}

export function verifyStockRuntimeImages(file: string, receipt: string, root = process.cwd()) {
  const images = verifyNativeAddonImages(file);
  const { report } = images;
  const qualification = qualifyStockAddon(root, report.base.revision, report.head.revision);
  assert.deepEqual(JSON.parse(fs.readFileSync(receipt, "utf8")), qualification);
  assert.equal(qualification.mode, "stock-runtime");
  // Source and lockfile changes are inputs, not a claim of identical machine code.
  // The independent builds must still use the same compiler, target and features.
  assert.deepEqual(
    report.differences.filter((field: string) => field !== "node-addon" && field !== "lock"),
    [],
  );
  for (const side of [report.base, report.head]) {
    const lockfile = execFileSync("git", ["show", `${side.revision}:Cargo.lock`], { cwd: root });
    assert.equal(createHash("sha256").update(lockfile).digest("hex"), side.lock);
  }
  return images;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [base, head, receipt, ...extra] = process.argv.slice(2);
  assert.ok(receipt && !extra.length, "expected BASE_SHA HEAD_SHA RECEIPT.json");
  const qualification = qualifyStockAddon(process.cwd(), base, head);
  fs.writeFileSync(receipt, JSON.stringify(qualification, null, 2) + "\n");
  console.log(qualification.mode);
}
