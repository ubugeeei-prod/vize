import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  compare,
  countSites,
  describe,
  inScope,
  inventoryColumns,
  inventoryPath,
  parseInventory,
  rustProductLines,
  scanTree,
} from "../../tools/ci/commands/temp-producers.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const header = inventoryColumns.join("\t");

function row(file: string, sites: number): string {
  return [file, String(sites), "purpose", "per run", "0 B", "the run", "0700", "owner"].join("\t");
}

test("every product temp site in the repository is listed with its exact count", () => {
  const rows = parseInventory(fs.readFileSync(path.join(repoRoot, inventoryPath), "utf8"));
  const findings = compare(scanTree(repoRoot), rows);
  assert.deepEqual(findings.map(describe), []);
});

test("cfg(test) items are not product code", () => {
  const source = [
    "fn product() { let _ = tempfile::tempdir(); }",
    "#[cfg(test)]",
    "use tempfile::TempDir;",
    "#[cfg(all(test, unix))]",
    "#[expect(",
    '    clippy::unwrap_used, reason = "tests"',
    ")]",
    "mod tests {",
    "    fn inner() {",
    "        let _ = tempfile::tempdir();",
    "    }",
    "}",
    "#[cfg(test)] fn helper() -> tempfile::TempDir { todo!() }",
    "fn after() { let _ = std::env::temp_dir(); }",
  ].join("\n");
  assert.deepEqual(rustProductLines(source), [
    "fn product() { let _ = tempfile::tempdir(); }",
    "fn after() { let _ = std::env::temp_dir(); }",
  ]);
  assert.equal(countSites("crates/a/src/lib.rs", source), 2);
});

test("comments and string literals are not sites", () => {
  const rust = [
    "// tempfile::tempdir() is avoided here",
    'let label = "tempfile::tempdir()";',
    "let dir = tempfile::Builder::new().tempdir()?; // the one site",
  ].join("\n");
  assert.equal(countSites("crates/a/src/lib.rs", rust), 1);
  const script = [
    "// os.tmpdir() is avoided here",
    'const label = "mkdtempSync(";',
    'const dir = fs.mkdtempSync(path.join(os.tmpdir(), "x-"));',
  ].join("\n");
  assert.equal(countSites("npm/a/src/index.ts", script), 1);
});

test("only product sources are in scope", () => {
  assert.deepEqual(
    [
      "crates/vize/src/commands/node_module.rs",
      "crates/vize/src/commands/check/runner/tests.rs",
      "crates/vize/src/commands/check/runner/shell_tests.rs",
      "crates/vize/tests/cli.rs",
      "crates/vize/build.rs",
      "npm/cli/src/cli.ts",
      "npm/cli/src/cli.test.ts",
      "npm/cli/scripts/release.ts",
      "npm/builder/vite-musea/src/art.fixtures.ts",
      "npm/linter/oxlint/src/test-support/fixture-dir.ts",
      "tools/ci/commands/temp-producers.ts",
    ].filter(inScope),
    ["crates/vize/src/commands/node_module.rs", "npm/cli/src/cli.ts"],
  );
});

test("a module declared under cfg(test) is skipped with its children", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-temp-producers-"));
  try {
    const files = new Map([
      ["crates/a/src/lib.rs", "mod product;\n#[cfg(test)]\nmod checks;\n"],
      ["crates/a/src/product.rs", "fn f() { let _ = tempfile::tempdir(); }\n"],
      ["crates/a/src/checks.rs", "fn f() { let _ = tempfile::tempdir(); }\n"],
      ["crates/a/src/checks/deep.rs", "fn f() { let _ = tempfile::tempdir(); }\n"],
    ]);
    for (const [file, source] of files) {
      fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
      fs.writeFileSync(path.join(root, file), source);
    }
    const found = scanTree(root, [...files.keys()]);
    assert.deepEqual([...found], [["crates/a/src/product.rs", 1]]);
    const unlisted = compare(found, parseInventory(`${header}\n`));
    assert.deepEqual(unlisted.map(describe), [
      `crates/a/src/product.rs: 1 unlisted temp site(s); add a row to ${inventoryPath}`,
    ]);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("listed counts must match the sites exactly", () => {
  const rows = parseInventory(`${header}\n${row("a.rs", 2)}\n${row("b.rs", 1)}\n`);
  assert.deepEqual(
    compare(
      new Map([
        ["a.rs", 3],
        ["c.rs", 1],
      ]),
      rows,
    ),
    [
      { path: "a.rs", listed: 2, found: 3 },
      { path: "b.rs", listed: 1, found: 0 },
      { path: "c.rs", listed: 0, found: 1 },
    ],
  );
  assert.equal(
    describe({ path: "b.rs", listed: 1, found: 0 }),
    `b.rs: no temp sites remain; delete its row from ${inventoryPath}`,
  );
});

test("the inventory format is strict", () => {
  assert.throws(() => parseInventory("path\tsites\n"), /inventory header must be/u);
  assert.throws(() => parseInventory(`${header}\na.rs\t1\n`), /needs 8 non-empty cells/u);
  assert.throws(() => parseInventory(`${header}\n${row("a.rs", 0)}\n`), /positive integer/u);
  assert.throws(
    () => parseInventory(`${header}\n${row("a.rs", 1)}\n${row("a.rs", 1)}\n`),
    /duplicate inventory row/u,
  );
  assert.throws(
    () => parseInventory(`${header}\n${row("b.rs", 1)}\n${row("a.rs", 1)}\n`),
    /sorted by path/u,
  );
});
