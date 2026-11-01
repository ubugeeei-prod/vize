import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { asArray, asRecord, asString, parseJson } from "../../json-value.ts";
import { originalBytes, type OriginalContract } from "./authority.ts";
import { cli, config, type Context } from "./install.ts";

export const semiCells = [
  { configured: false, flags: [], semi: false },
  { configured: false, flags: ["--semi"], semi: true },
  { configured: false, flags: ["--semi=true"], semi: true },
  { configured: true, flags: [], semi: true },
  { configured: true, flags: ["--semi=false"], semi: false },
  { configured: true, flags: ["--no-semi"], semi: false },
] as const;
export const invalidSemiFlags = [
  ["--semi=invalid"],
  ["--semi", "--no-semi"],
  ["--no-semi", "--semi=false"],
];
export function semiCases(original: OriginalContract) {
  const corpus = asRecord(
    parseJson(
      originalBytes(
        original,
        "tests/_fixtures/differential/formatter-regressions/semicolon-cli-6098/cases.json",
      ).toString("utf8"),
    ),
  );
  assert.equal(corpus.schema, "vize.formatter-semicolon-cli-6098");
  assert.deepEqual(corpus.reference, { package: "prettier", version: "3.8.3" });
  const cases = asArray(corpus.cases).map((value) => {
    const row = asRecord(value);
    return {
      file: asString(row.file),
      parser: asString(row.parser),
      source: asString(row.source),
      withSemicolons: asString(row.withSemicolons),
      withoutSemicolons: asString(row.withoutSemicolons),
    };
  });
  assert.deepEqual(
    cases.map((row) => [row.file, row.parser]),
    [
      ["Original雪🌸.js", "babel"],
      ["Original雪🌸.ts", "typescript"],
    ],
  );
  for (const row of cases) assert.notEqual(row.withSemicolons, row.withoutSemicolons);
  return cases;
}

export function runSemi(context: Context): void {
  const cases = semiCases(context.original),
    configPath = join(context.root, "vite.config.mjs");
  const results: unknown[] = [];
  for (const row of cases) {
    for (const [index, cell] of semiCells.entries()) {
      const label = "semi-" + row.parser + "-" + index,
        path = join(context.root, row.file);
      writeFileSync(path, row.source);
      const authoredConfig = config(context.root, {
        semi: cell.configured,
        singleQuote: true,
        printWidth: 73,
      });
      const expected = cell.semi ? row.withSemicolons : row.withoutSemicolons;
      const files = [row.file, "vite.config.mjs"];
      context.record.json(label + ".input.json", {
        row,
        cell,
        expected,
        configRawBase64: authoredConfig.toString("base64"),
      });
      const run = (action: string, phase: string, status: number) =>
        cli(
          context,
          label + "-" + phase,
          ["--config", "vite.config.mjs", action, ...cell.flags, row.file],
          status,
          files,
        );
      run("--check", "dirty-check", 1);
      assert.equal(readFileSync(path, "utf8"), row.source);
      assert.deepEqual(readFileSync(configPath), authoredConfig);
      for (const attempt of [0, 1]) {
        run("--write", "write-" + attempt, 0);
        assert.equal(readFileSync(path, "utf8"), expected);
        assert.deepEqual(readFileSync(configPath), authoredConfig);
      }
      run("--check", "fixed-point", 0);
      assert.equal(readFileSync(path, "utf8"), expected);
      assert.deepEqual(readFileSync(configPath), authoredConfig);
      results.push({ file: row.file, cell, completeOutput: expected, fixedPoint: true });
      context.record.json("semi-results.json", results);
    }
  }
  assert.equal(results.length, 12);
  const row = cases[0]!;
  writeFileSync(join(context.root, row.file), row.source);
  const authoredConfig = config(context.root, { semi: false });
  for (const [index, flags] of invalidSemiFlags.entries()) {
    const packet = cli(
      context,
      "semi-refusal-" + index,
      ["--config", "vite.config.mjs", "--write", ...flags, row.file],
      2,
      [row.file, "vite.config.mjs"],
    );
    assert.equal(packet.stdout, "");
    assert.notEqual(packet.stderr, "");
    assert.equal(readFileSync(join(context.root, row.file), "utf8"), row.source);
    assert.deepEqual(readFileSync(configPath), authoredConfig);
  }
  context.record.json("semi-qualification.json", {
    cells: results.length,
    originalCommands: 48,
    refusalCommands: invalidSemiFlags.length,
    reference:
      "frozen independently executed Prettier3.8.3 whole bytes; not reexecuted in this public role",
  });
}
