// #6858: the TSRX Vue profile is measured against one pinned reference compiler.
// These tests are the drift check: the committed corpus, the catalog pin and the
// decision record must all name the compiler that is actually installed.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";

import {
  corpusFile,
  corpusFiles,
  installedCompiler,
  recordCorpus,
  repoRoot,
} from "./tsrx-oracle/reference.ts";

const committed = JSON.parse(readFileSync(corpusFile, "utf8"));

test("the committed TSRX reference corpus matches the pinned compiler", () => {
  assert.deepEqual(recordCorpus(), committed);
});

test("every corpus module has exactly one reference row", () => {
  assert.deepEqual(
    committed.rows.map((row: { file: string }) => row.file),
    corpusFiles(),
  );
  for (const row of committed.rows) {
    const failed = row.file.includes("/errors/");
    assert.equal(row.error !== null, failed, `${row.file}: errors/ rows, and only they, fail`);
    if (failed) assert.match(row.error.code ?? "", /^TSRX\d{4}$/u, row.file);
  }
});

test("the catalog and the decision record pin the installed compiler", () => {
  const installed = installedCompiler();
  const workspace = readFileSync(join(repoRoot, "pnpm-workspace.yaml"), "utf8");
  const catalog = /^ {2}tsrx-oracle:\n((?: {4}.+\n)+)/mu.exec(workspace)?.[1] ?? "";
  const pinned = Object.fromEntries(
    [...catalog.matchAll(/^ {4}"([^"]+)": "([^"]+)"$/gmu)].map(([, name, version]) => [
      name,
      version,
    ]),
  );
  assert.deepEqual(pinned, installed);
  const record = readFileSync(
    join(repoRoot, "docs/davinci/decisions/2026-11-09-6858-tsrx-vue-profile.md"),
    "utf8",
  );
  for (const [name, version] of Object.entries(installed)) {
    assert.match(
      record,
      new RegExp(
        `\\| \`${name.replace("/", "\\/")}\`\\s+\\| ${version.replaceAll(".", "\\.")}\\s`,
        "u",
      ),
      `${name}@${version} in the decision record`,
    );
  }
});
