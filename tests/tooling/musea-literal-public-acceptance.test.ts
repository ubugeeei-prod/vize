import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";
import { parse } from "yaml";
import {
  adaptDriver,
  assertSuiteCensus,
  digest,
  drivers,
  materialize,
} from "../../tools/support/release/musea_literal_acceptance/fixtures.ts";

const sourceHead = "15e116fa258c7aaba5c6fdd2456442535605abe8";
const authority = new URL("../_fixtures/release/musea-literal-443/", import.meta.url);
const manifest = JSON.parse(fs.readFileSync(new URL("manifest.json", authority), "utf8")) as {
  source: string;
  files: { source: string; file: string; sha256: string }[];
};
assert.equal(manifest.source, sourceHead);
assert.equal(manifest.files.length, 5);
assert.equal(new Set(manifest.files.map((row) => row.source)).size, 5);
const originals = new Map(
  manifest.files.map((row) => {
    assert.equal(path.basename(row.file), row.file);
    const bytes = fs.readFileSync(new URL(row.file, authority));
    assert.equal(digest(bytes), row.sha256, `complete original H bytes: ${row.source}`);
    return [row.source, bytes];
  }),
);
const original = (file: string) => {
  const bytes = originals.get(file);
  assert.ok(bytes, `retained complete H fixture: ${file}`);
  return bytes;
};

void test("literal public adapters preserve both complete original H laws and every assertion", () => {
  for (const file of drivers) {
    const bytes = original(file).toString();
    const adapted = adaptDriver(file, bytes, "/tmp/consumer");
    let reversed = adapted.bytes;
    for (const replacement of [...adapted.replacements].reverse()) {
      assert.equal(reversed.split(replacement.to).length, 2);
      reversed = reversed.replace(replacement.to, replacement.from);
    }
    assert.equal(reversed, bytes);
    const assertions = (value: string) =>
      value.split("\n").filter((line) => /^\s*assert\./u.test(line));
    assert.deepEqual(assertions(adapted.bytes), assertions(bytes));
  }
  assert.throws(
    () =>
      adaptDriver(
        drivers[0],
        original(drivers[0]).toString().replace('import vize from "../../vite/src/index.ts";', ""),
        "/tmp/consumer",
      ),
    /unique adapter source/u,
  );
});

void test("literal preparation rejects omitted fixtures and substituted whole source laws", (t) => {
  const root = fs.mkdtempSync(path.join(tmpdir(), "musea-literal-prepare-law-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const corpus = "tests/_fixtures/differential/musea/literal-variant-keys.json";
  const files = [
    corpus,
    ...JSON.parse(original(corpus).toString()).sources.map((row: { path: string }) => row.path),
  ];
  assert.throws(() => materialize(path.join(root, "omitted"), files.slice(0, 2), original));
  assert.throws(
    () =>
      materialize(path.join(root, "substituted"), files, (file) =>
        file === drivers[1]
          ? Buffer.from(original(file).toString().replace("assert.deepEqual(errors, []);", ""))
          : original(file),
      ),
    /complete original release H driver/u,
  );
  const result = materialize(path.join(root, "whole"), files, original);
  assert.equal(result.records.length, 5);
  assert.equal(result.adapters.length, 2);
});

void test("literal terminal census rejects zero-work skip duplicate and missing summaries", () => {
  const complete = "# tests 2\n# pass 2\n# fail 0\n# cancelled 0\n# skipped 0\n# todo 0\n";
  assertSuiteCensus(complete);
  for (const invalid of [
    complete.replace("# tests 2", "# tests 0"),
    complete.replace("# skipped 0", "# skipped 1"),
    complete + "# pass 2\n",
    complete.replace("# pass 2\n", ""),
  ])
    assert.throws(() => assertSuiteCensus(invalid));
});

void test("literal workflow exports a real runner path and propagates tee pipeline failure", (t) => {
  const workflow = parse(
    fs.readFileSync(".github/workflows/release-musea-literal-acceptance.yml", "utf8"),
  );
  const job = workflow.jobs["public-browser"];
  assert.equal(job.defaults.run.shell, "bash");
  for (const value of Object.values(job.env))
    assert.doesNotMatch(String(value), /\$\{\{\s*runner\./u);
  const root = fs.mkdtempSync(path.join(tmpdir(), "musea-literal-bootstrap-law-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const environment = {
    PATH: process.env.PATH,
    GITHUB_REPOSITORY: "ubugeeei-prod/vize",
    GITHUB_REF: "refs/heads/main",
    RUNNER_TEMP: root,
    GITHUB_ENV: path.join(root, "environment"),
    GITHUB_RUN_ID: "41",
    GITHUB_RUN_ATTEMPT: "2",
  };
  const execute = (
    run: string,
    env: typeof environment & { NODE_OPTIONS?: string } = environment,
  ) =>
    spawnSync("bash", ["--noprofile", "--norc", "-e", "-o", "pipefail", "-c", run], {
      env,
      encoding: "utf8",
    });
  assert.equal(execute(job.steps[0].run).status, 0);
  assert.equal(
    fs.readFileSync(environment.GITHUB_ENV, "utf8"),
    `PUBLIC_OUTPUT=${root}/vize-public-musea-literal-41-2\n`,
  );
  assert.equal(execute(job.steps[0].run, { ...environment, NODE_OPTIONS: "forbidden" }).status, 1);
  assert.equal(
    execute(job.steps[0].run, { ...environment, GITHUB_REF: "refs/heads/unreviewed" }).status,
    1,
  );
  assert.equal(execute("(exit 7) | tee /dev/null").status, 7);
  const installs = job.steps.filter((step: { run?: string }) =>
    step.run?.includes("npm install --ignore-scripts"),
  );
  assert.equal(installs.length, 1);
});
