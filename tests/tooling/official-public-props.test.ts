import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  assertOfficialDiagnostics,
  loadOfficialFixture,
  officialCases,
  officialRunOptions,
} from "../../tools/support/compat/content-mapper/official-fixtures.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const fixturePath = "tests/_fixtures/content-mapper/escaton-public-props-9498.json";
const fixture = loadOfficialFixture(path.join(root, fixturePath));
const original = fixture.originals[0];
const cases = officialCases(fixture);
// The source-built official TS7.1/Vue3.5.20 run 39146091720 captured these two
// streams. Their fixture rows deliberately use code/term/authored-range guards;
// do not give tests stronger whole-stream oracles than the actual public actor.
const additionalSourceStreams: Record<string, string> = {
  "public-props-9498-unknown-prop-strict":
    "App.vue(4,46): error TS2353: Object literal may only specify known properties, and '\"definitelyNotAProp\"' does not exist in type '__VizeComponentCheckProps<__VizeStaticEmitProps, __VizePublicComponentAttrs & { abbr?: unknown; about?: unknown; \"accent-height\"?: unknown; accentHeight?: unknown; accept?: unknown; acceptcharset?: unknown; ... 661 more ...; zoomAndPan?: unknown; } & ...>'.\n",
  "public-props-9498-wrong-listener-payload":
    "App.vue(5,53): error TS2345: Argument of type 'number' is not assignable to parameter of type 'string'.\n",
};
const result = (stdout: string, status: number, stderr = "") => ({
  status,
  signal: null,
  error: null,
  stdout: { bytes: Buffer.byteLength(stdout) },
  stderr: { bytes: Buffer.byteLength(stderr) },
  stdoutText: stdout,
  stderrText: stderr,
});

test("public-prop reproduction changes only the listener or the invalid prop value", () => {
  const fixture = loadOfficialFixture(
    new URL("../_fixtures/content-mapper/escaton-public-props-9498.json", import.meta.url).pathname,
  );
  assert.equal(fixture.baselineVize, "0.467.0");
  const original = fixture.originals[0];
  assert.equal(original.issue, 9498);
  assert.deepEqual(original.baselineCodes, [2353]);
  assert.deepEqual(original.expectedCodes, []);
  const control = (name: string) => {
    const row = fixture.controls.find((row) => row.name === `public-props-9498-${name}`);
    assert.ok(row, name);
    return row;
  };
  const withoutListener = control("without-listener");
  assert.deepEqual(withoutListener.files, {
    ...original.files,
    "App.vue": original.files["App.vue"].replace(' @close="() => {}"', ""),
  });
  assert.equal(withoutListener.stdout, "");
  assert.equal(withoutListener.status, 0);
  for (const [name, owner] of [
    ["wrong-allowed-prop", withoutListener],
    ["wrong-allowed-prop-listener", original],
  ] as const) {
    const invalid = control(name);
    assert.deepEqual(invalid.files, {
      ...owner.files,
      "App.vue": owner.files["App.vue"].replace(':hidden="true"', ":hidden=\"'wrong'\""),
    });
    assert.equal(invalid.status, 2);
    assert.ok(invalid.codes.every((code) => code === 2322));
  }
  const defaultAttrs = control("unknown-prop");
  const strictAttrs = control("unknown-prop-strict");
  assert.deepEqual(defaultAttrs.codes, []);
  assert.equal(defaultAttrs.status, 0);
  assert.equal(defaultAttrs.stdout, "");
  assert.equal(defaultAttrs.stderr, "");
  const strictConfig = JSON.parse(defaultAttrs.files["tsconfig.json"]);
  strictConfig.vueCompilerOptions.checkUnknownProps = true;
  assert.deepEqual(strictAttrs.files, {
    ...defaultAttrs.files,
    "tsconfig.json": `${JSON.stringify(strictConfig, null, 2)}\n`,
  });
  assert.deepEqual(strictAttrs.codes, [2353]);
  assert.equal(strictAttrs.status, 2);
  assert.equal(officialCases(fixture).length, 11);
  const workflow = fs.readFileSync(
    new URL("../../.github/workflows/content-mapper-conformance.yml", import.meta.url),
    "utf8",
  );
  const fixturePath = "tests/_fixtures/content-mapper/escaton-public-props-9498.json";
  assert.equal(workflow.split(fixturePath).length - 1, 3);
  assert.ok(workflow.includes(`--fixture ${fixturePath}`));
});

test("public9498 retains the canonical eleven-case selection without default suites", () => {
  assert.equal(
    createHash("sha256")
      .update(fs.readFileSync(path.join(root, fixturePath)))
      .digest("hex"),
    "fdcc0187a0042d5fd59e9e75366467df09bbd6b3e91b262645bbd7eff34b7fd6",
  );
  // A parser-only synthetic tuple gives no release or installed-product credit.
  const options = officialRunOptions(
    [
      "--candidate-only",
      "--explicit-fixtures-only",
      "--public-version",
      "0.1.0",
      "--public-inputs",
      "frozen-public-inputs.json",
      "--fixture",
      fixturePath,
    ],
    root,
    path.join(root, "tests/_fixtures/content-mapper/escaton-css-options.json"),
  );
  assert.equal(options.candidateOnly, true);
  assert.equal(options.baselineOnly, false);
  assert.deepEqual(options.fixturePaths, [path.join(root, fixturePath)]);
  assert.match(original.files["env.d.ts"], /declare module ["']@vue\/runtime-core["']/u);
  assert.match(original.files["env.d.ts"], /hidden\?: boolean/u);
  assert.deepEqual(
    cases.map(({ name, codes, status }) => [name, codes, status]),
    [
      ["issue-9498", [], 0],
      ["public-props-9498-without-listener", [], 0],
      ["public-props-9498-wrong-allowed-prop-listener", [2322, 2322, 2322], 2],
      ["public-props-9498-wrong-allowed-prop", [2322, 2322], 2],
      ["public-props-9498-unknown-prop", [], 0],
      ["public-props-9498-unknown-prop-strict", [2353], 2],
      ["public-props-9498-custom-props-listener", [], 0],
      ["public-props-9498-wrong-custom-prop", [2322, 2322], 2],
      ["public-props-9498-vnode-props-listener", [], 0],
      ["public-props-9498-generic-listener", [], 0],
      ["public-props-9498-wrong-listener-payload", [2345], 2],
    ],
  );
});

test("published9498 keeps complete typed negative streams and refuses process or mapping failures", () => {
  for (const row of cases) {
    const out = row.stdout ?? additionalSourceStreams[row.name];
    assert.equal(typeof out, "string", row.name);
    const status = row.status ?? (row.codes.length ? 2 : 0);
    const observed = result(out, status);
    assertOfficialDiagnostics(row, original, false, observed);
    for (const invalid of [
      result(out, status, "unexpected stderr"),
      result(out, status === 0 ? 2 : 0),
      { ...observed, signal: "SIGTERM" },
      { ...observed, error: new Error("compiler failed") },
      result("error TS2353: Unmapped virtual property.\n" + out, 2),
    ])
      assert.throws(() => assertOfficialDiagnostics(row, original, false, invalid));
    if (row.codes.length && row.stdout !== undefined) {
      const relocated = out.replace(
        /\((\d+),(\d+)\)/u,
        (_, line: string, column: string) => `(${line},${Number(column) + 1})`,
      );
      assert.notEqual(relocated, out);
      assert.throws(() =>
        assertOfficialDiagnostics(row, original, false, result(relocated, status)),
      );
      assert.throws(() =>
        assertOfficialDiagnostics(row, original, false, result(out + out, status)),
      );
    }
  }
});

test("public9498 acceptance laws are enrolled in ordinary source and full merge tooling", () => {
  const file = "tests/tooling/official-public-props.test.ts";
  const requests: Array<{ input: string; tier: string }> = [];
  for (const tier of ["pr", "merge"]) {
    for (const input of [file, fixturePath]) requests.push({ input, tier });
  }
  const plans: unknown = JSON.parse(
    execFileSync(
      process.execPath,
      [
        "--input-type=module",
        "--eval",
        `import { planToolingTests } from './tools/support/compat/github/plan-tooling-tests.mjs';
         const requests = JSON.parse(process.argv[1]);
         console.log(JSON.stringify(requests.map(({input, tier}) =>
           planToolingTests([input], {tier, cwd: process.cwd()}).tests)));`,
        JSON.stringify(requests),
      ],
      { cwd: root, encoding: "utf8" },
    ),
  );
  assert.ok(Array.isArray(plans));
  assert.equal(plans.length, requests.length);
  for (const [index, tests] of plans.entries()) {
    assert.ok(Array.isArray(tests) && tests.every((name) => typeof name === "string"));
    assert.ok(tests.includes(file), JSON.stringify(requests[index]));
  }
});
