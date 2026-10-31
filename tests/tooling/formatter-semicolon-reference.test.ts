import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";

type Case = {
  file: string;
  parser: "babel" | "typescript";
  source: string;
  withSemicolons: string;
  withoutSemicolons: string;
};
const corpus = JSON.parse(
  fs.readFileSync(
    new URL(
      "../_fixtures/differential/formatter-regressions/semicolon-cli-6098/cases.json",
      import.meta.url,
    ),
    "utf8",
  ),
) as {
  schema: string;
  reference: { package: string; version: string };
  cases: Case[];
};
const require = createRequire(
  new URL("../../tools/benchmarks/scripts/package.json", import.meta.url),
);
const prettier = require("prettier") as {
  version: string;
  format(
    source: string,
    options: { parser: string; semi: boolean; singleQuote: boolean; printWidth: number },
  ): Promise<string>;
};

test("new JS and TS semicolon controls match complete pinned independent outputs and fixed points", async () => {
  assert.equal(corpus.schema, "vize.formatter-semicolon-cli-6098");
  assert.deepEqual(corpus.reference, { package: "prettier", version: "3.8.3" });
  assert.equal(prettier.version, corpus.reference.version);
  assert.deepEqual(
    corpus.cases.map((row) => row.parser),
    ["babel", "typescript"],
  );
  for (const row of corpus.cases) {
    for (const semi of [true, false]) {
      const options = { parser: row.parser, semi, singleQuote: true, printWidth: 73 };
      const expected = semi ? row.withSemicolons : row.withoutSemicolons;
      const actual = await prettier.format(row.source, options);
      assert.equal(actual, expected, `${row.file}: complete output semi=${semi}`);
      assert.equal(await prettier.format(actual, options), actual, `${row.file}: fixed point`);
      assert.notEqual(row.withSemicolons, row.withoutSemicolons);
    }
  }
});
