import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import {
  afterProfile,
  commandsFor,
  filesFor,
  limitationsFor,
  requireProfile,
  sha256,
  subsetFor,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/contract.ts";
import {
  assessTsrs,
  capture,
  recorder,
  type ProcessOutput,
} from "../../tools/benchmarks/scripts/public-typechecker-q16/execute.ts";
import { requireOwnedDispatch } from "../../tools/benchmarks/scripts/public-typechecker-q16/run.ts";

const selected = subsetFor("R77"),
  project = selected.projects[0]!;
const originalHashes = {
  "tsconfig.json": "44ba7c3bf8f56fc662ff6729db3dc841c9b5646c76db71eecddcd17eb1889181",
  "src/useBox.ts": "8b69196f7909119360e50ac2d72dba8705c29d50155b8d7e6843c27884ec5387",
  "src/StaticRef.vue": "a439b4747a8cc8b5c1adf9562d5412d61bbfdfe605886c945f9c2ecd0bf4c7c6",
};
const line =
  "src/StaticRef.vue(7,7): error TS6133: 'trueUnused' is declared but its value is never read.\n";

test("R77 keeps the exact three reported files and only the independent authored-unused mutation", () => {
  assert.equal(selected.projects.length, 1);
  assert.equal(project.issue, 11377);
  assert.deepEqual(
    project.states.map((state) => state.id),
    [
      "11377/original",
      "11377/authored-unused",
      "11377/repair",
      "11377/negative-restoration",
      "11377/final-original-restoration",
    ],
  );
  const original = project.states[0]!,
    negative = project.states[1]!;
  for (const [path, hash] of Object.entries(originalHashes))
    assert.equal(sha256(original.files.find((file) => file.path === path)!.source), hash);
  assert.deepEqual(project.states[2]!.files, original.files);
  assert.deepEqual(project.states[3]!.files, negative.files);
  assert.deepEqual(project.states[4]!.files, original.files);
  for (const file of negative.files) {
    const before = original.files.find((row) => row.path === file.path)!;
    assert.equal(
      file.source,
      file.path === "src/StaticRef.vue"
        ? before.source.replace("</script>", "const trueUnused = 1;\n</script>")
        : before.source,
    );
  }
  assert.equal(
    sha256(negative.files.find((file) => file.path === "src/StaticRef.vue")!.source),
    "290b1b55ca37f1f8e690d1963d3402c367517c16f64448b56ee5e034b5728b14",
  );
  assert.deepEqual(
    project.states.map((state) => state.expectedTsrs),
    [
      { status: 0, stdout: "", stderr: "" },
      { status: 2, stdout: line, stderr: "" },
      { status: 0, stdout: "", stderr: "" },
      { status: 2, stdout: line, stderr: "" },
      { status: 0, stdout: "", stderr: "" },
    ],
  );
});

test("R77 authenticates the containing release separately from fix identity and substitutes only Vize", () => {
  const profile = requireProfile(afterProfile);
  assert.equal(profile.subset, "R77");
  assert("fix" in profile);
  assert.equal(profile.fix, "56635d7cfb27718ec8b40e67309c71da5b9a3803");
  assert.equal(profile.source, "155820d442a6250be45eb52c89c455e343f07905");
  assert.notEqual(profile.fix, profile.source);
  assert.equal(profile.version, "0.648.0");
  assert.equal(profile.capsule, "aa2694a717c6749d85ce694eab8b5b0812fdab7f");
  assert.throws(() => requireProfile({ ...profile, fix: profile.source }));
  assert.throws(() => requireProfile(null));
  for (const state of project.states) {
    const files = filesFor(state, profile),
      pkg = JSON.parse(files[0]!.source);
    assert.deepEqual(pkg.devDependencies, {
      vize: "0.648.0",
      tsrs: "0.20.1-ts7.1.0-beta",
      vue: "3.5.43",
    });
    for (let index = 1; index < files.length; index++)
      assert.deepEqual(files[index], state.files[index]);
    assert.equal(
      files[0]!.source,
      state.files[0]!.source.replace('"vize": "0.641.0"', '"vize": "0.648.0"'),
    );
  }
});

test("R77 uses original TSRS flags without reference tools and preserves older role commands", () => {
  assert.deepEqual(commandsFor("R77"), {
    install: ["npm", "i"],
    tsrs: ["npx", "tsrs", "--runExternalCode", "-p", "tsconfig.json", "--pretty", "false"],
    reference: null,
  });
  assert.deepEqual(commandsFor("D61").tsrs, [
    "npx",
    "tsrs",
    "--runExternalCode",
    "--noEmit",
    "-p",
    "tsconfig.json",
  ]);
  assert.deepEqual(commandsFor("E90-91").tsrs, [
    "npx",
    "tsrs",
    "--fast",
    "--runExternalCode",
    "-p",
    "tsconfig.json",
  ]);
  assert.match(limitationsFor("R77").host, /macOS26\.4\.1 arm64/u);
  assert.match(limitationsFor("R77").manager, /pnpm12\.3\.4/u);
  assert.match(limitationsFor("D61").host, /Windows11/u);
  requireOwnedDispatch({
    GITHUB_EVENT_NAME: "workflow_dispatch",
    GITHUB_REPOSITORY: "ubugeeei-prod/vize",
    GITHUB_REF: "refs/heads/test/published-static-template-ref-11377-20261102",
  });
  assert.throws(() =>
    requireOwnedDispatch({
      GITHUB_EVENT_NAME: "workflow_dispatch",
      GITHUB_REPOSITORY: "ubugeeei-prod/vize",
      GITHUB_REF: "refs/heads/test/unowned",
    }),
  );
});

test("R77 retains whole literal negative streams and refuses lost or altered authored findings", () => {
  const root = mkdtempSync(join(tmpdir(), "public-r77-control-")),
    record = recorder(root),
    state = project.states[1]!;
  const output = (stdout: string, status = 2): ProcessOutput => ({
    status,
    signal: null,
    pid: 123,
    error: null,
    stdout: Buffer.from(stdout),
    stderr: Buffer.alloc(0),
    capacityLimited: false,
  });
  try {
    let index = 0;
    const packet = (stdout: string, status = 2) =>
      capture(
        record,
        "case-" + index++,
        ["mock-tsrs"],
        root,
        () => {},
        () => output(stdout, status),
      );
    assert.equal(assessTsrs(state, packet(line)).completeTsrsVectorMatchesSourceIntent, true);
    for (const wrong of [
      "",
      line + line,
      line.replace("(7,7)", "(5,7)"),
      line.replace("TS6133", "TS2322"),
    ])
      assert.throws(() => assessTsrs(state, packet(wrong)));
    assert.throws(() => assessTsrs(state, packet(line, 1)));
    assert.equal(readFileSync(join(root, "case-1.stdout.raw"), "utf8"), "");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("R77 reuses strict installed route and same-run contracts without source or companion changes", () => {
  const run = readFileSync(
    new URL("../../tools/benchmarks/scripts/public-typechecker-q16/run.ts", import.meta.url),
    "utf8",
  );
  assert(run.includes('commands.reference === null ? ["tsrs"] : undefined'));
  const workflow = readFileSync(
    new URL("../../.github/workflows/public-typechecker-consumer.yml", import.meta.url),
    "utf8",
  );
  assert(workflow.includes("node --test tests/tooling/public-typechecker-q16*.test.ts"));
  assert(workflow.includes("refs/heads/test/published-static-template-ref-11377-20261102"));
});
