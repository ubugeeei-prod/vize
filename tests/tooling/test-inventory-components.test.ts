import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  attachSourceCoverage,
  collectInventory,
  renderInventoryMarkdown,
} from "../../tools/benchmarks/scripts/test-inventory.ts";

test("component inventory separates static declarations from measured execution", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-component-inventory-"));
  const write = (file: string, content: string) => {
    const absolute = path.join(root, file);
    fs.mkdirSync(path.dirname(absolute), { recursive: true });
    fs.writeFileSync(absolute, content);
  };
  try {
    write("crates/formatter/Cargo.toml", '[package]\nname = "formatter"\n');
    write("crates/formatter/src/lib.rs", "#[test]\nfn preserves_input() {}\n");
    write("crates/lsp/Cargo.toml", '[package]\nname = "lsp"\n');
    write("npm/editor/package.json", '{"name":"editor"}');
    write("npm/editor/provider.test.ts", 'test("opens document", () => {});');
    write("tests/tooling/workspace.test.ts", 'test("cross component", () => {});');
    write("node_modules/external/fake.test.ts", 'test("ignored", () => {});');
    const inventory = collectInventory(root);
    assert.deepEqual(inventory.components, [
      { component: "crates/formatter", files: 1, cases: 1 },
      { component: "crates/lsp", files: 0, cases: 0 },
      { component: "npm/editor", files: 1, cases: 1 },
      { component: "tests/tooling", files: 1, cases: 1 },
    ]);
    assert.equal(inventory.totalCases, 3);
    const before = renderInventoryMarkdown(inventory);
    assert.match(before, /crates\/lsp \| 0 \| 0 \| unmeasured/);
    const measuredFile = (filename: string, count: number, covered: number) => ({
      filename,
      summary: Object.fromEntries(
        ["lines", "regions", "functions"].map((key) => [key, { count, covered }]),
      ),
    });
    attachSourceCoverage(
      inventory,
      {
        type: "llvm.coverage.json.export",
        data: [
          {
            files: [
              measuredFile("/checkout/crates/formatter/src/one.rs", 1, 1),
              measuredFile("C:\\checkout\\crates\\formatter\\src\\two.rs", 9, 0),
              measuredFile("/checkout/crates/lsp/src/provider.rs", 5, 0),
              measuredFile("/checkout/davinci/removed_component/src/lib.rs", 2, 1),
              measuredFile("/checkout/vendor/engine/src/lib.rs", 100, 100),
            ],
          },
        ],
      },
      "a".repeat(40),
    );
    assert.equal(inventory.totalCases, 3, "instrumentation must not become static cases");
    assert.deepEqual(inventory.components[0].coverage?.lines, { count: 10, covered: 1 });
    const markdown = renderInventoryMarkdown(inventory);
    assert.match(markdown, /1\/10 \(10\.00%\)/, "weight totals instead of averaging files");
    assert.match(markdown, /crates\/lsp \| 0 \| 0 \| 0\/5 \(0\.00%\)/);
    assert.match(markdown, /npm\/editor \| 1 \| 1 \| unmeasured/);
    assert.match(markdown, /davinci\/removed_component \| 0 \| 0 \| 1\/2/);
    assert.match(markdown, /coverage describes this separate commit/);
    assert.ok(!markdown.includes("vendor/engine"));
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("source coverage refuses absent provenance and malformed metric totals", () => {
  const inventory = {
    totalCases: 0,
    totalFiles: 0,
    areas: [],
    groups: [],
    components: [],
  };
  const report = (metric: unknown) => ({
    type: "llvm.coverage.json.export",
    data: [
      {
        files: [
          {
            filename: "crates/lsp/src/lib.rs",
            summary: { lines: metric, regions: metric, functions: metric },
          },
        ],
      },
    ],
  });
  assert.throws(() => attachSourceCoverage(inventory, report({}), ""), /full commit SHA/);
  assert.throws(() => attachSourceCoverage(inventory, {}, "a".repeat(40)), /LLVM|llvm-cov/);
  for (const metric of [
    { count: 10, covered: 11 },
    { count: -1, covered: 0 },
    { count: 10, covered: 0.5 },
    { count: 10 },
  ]) {
    assert.throws(
      () => attachSourceCoverage(inventory, report(metric), "a".repeat(40)),
      /Invalid LLVM lines totals/,
    );
  }
});
