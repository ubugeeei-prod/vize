import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

test("Rust inventory retains explicitly registered custom harnesses and ordinary tests", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-custom-harness-inventory-"));
  const write = (file: string, content: string) => {
    const absolute = path.join(root, file);
    fs.mkdirSync(path.dirname(absolute), { recursive: true });
    fs.writeFileSync(absolute, content);
  };
  try {
    write(
      "crates/sample/Cargo.toml",
      '[package]\nname = "sample"\nversion = "0.1.0"\n' +
        '[[test]]\nname = "cold_budget"\nharness = false\n' +
        '[[test]]\nname = "custom_path"\npath = "checks/window.rs"\nharness = false\n' +
        '[[test]]\nname = "missing_entry"\nharness = false\n' +
        '[[test]]\nname = "regular"\n' +
        '[[bench]]\nname = "bench_only"\npath = "checks/bench.rs"\nharness = false\n',
    );
    write(
      "crates/sample/tests/cold_budget.rs",
      "fn main() { check_budget(); }\nfn check_budget() {}\n",
    );
    write("crates/sample/checks/window.rs", "fn main() {}\n");
    write("crates/sample/tests/missing_entry.rs", "// fn main() {}\nfn helper() {}\n");
    write("crates/sample/tests/regular.rs", "#[test]\nfn ordinary_case() {}\n");
    write("crates/sample/checks/bench.rs", "fn main() {}\n");
    write("crates/sample/checks/unregistered.rs", "fn main() {}\n");

    const inventoryPath = path.join(root, "inventory.json");
    const script = fileURLToPath(
      new URL("../../tools/benchmarks/scripts/test-inventory.mjs", import.meta.url),
    );
    execFileSync(process.execPath, [script, "--json", inventoryPath], { cwd: root });
    const inventory = JSON.parse(fs.readFileSync(inventoryPath, "utf8")) as {
      totalCases: number;
      groups: Array<{
        area: string;
        runner: string;
        file: string;
        count: number;
        tests: Array<{ name: string }>;
      }>;
    };
    const groups = inventory.groups.filter((group: { area: string }) => group.area === "Rust");
    assert.equal(groups.length, 3);
    assert.equal(inventory.totalCases, 3);
    assert.deepEqual(
      groups.map((group: { file: string; count: number; tests: Array<{ name: string }> }) => ({
        file: group.file,
        count: group.count,
        names: group.tests.map((entry) => entry.name),
      })),
      [
        { file: "crates/sample/checks/window.rs", count: 1, names: ["custom_path"] },
        { file: "crates/sample/tests/cold_budget.rs", count: 1, names: ["cold_budget"] },
        { file: "crates/sample/tests/regular.rs", count: 1, names: ["ordinary_case"] },
      ],
    );
    assert.equal(groups[0].runner, "cargo test (custom harness)");
    assert.equal(groups[2].runner, "cargo test");
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
