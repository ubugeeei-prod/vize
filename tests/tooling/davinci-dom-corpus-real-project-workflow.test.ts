import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import {
  corpusEvidenceLines,
  expectedDomOutputComparisons,
  expectedGitlinks,
  hydrateCorpus,
  parseCorpusEvidence,
  parseFixtureGitlinks,
  parseOldErrorReasons,
  validateCorpusEvidence,
  verdictFor,
} from "../../tools/fixtures/davinci-dom-corpus-workflow.mjs";
import { findStep, readRealProjectMatrixWorkflow } from "./support/real-project-matrix-workflow.ts";

const helperSource = readFileSync("tools/fixtures/davinci-dom-corpus-workflow.mjs", "utf8");

test("real-project workflow carries a full-canonical S2 DOM corpus job", () => {
  const workflow = readRealProjectMatrixWorkflow();
  const job = workflow.jobs?.["davinci-dom-corpus"];
  assert.ok(job, "missing davinci-dom-corpus job");
  const steps = job.steps ?? [];

  assert.equal(job.name, "s2 dom corpus");
  assert.equal(job["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  assert.equal(job["timeout-minutes"], 120);
  assert.equal(
    job.env?.VIZE_DAVINCI_DOM_CORPUS_MODE,
    "${{ inputs.davinci_dom_corpus_mode || 'enforce' }}",
  );

  const checkout = steps.find((step) => step.uses?.startsWith("actions/checkout@"));
  assert.match(checkout?.uses ?? "", /de0fac2e4500dabe0009e67214ff5f5447ce83dd/);
  assert.deepEqual(checkout?.with, { "persist-credentials": false });
  assert.ok(steps.some((step) => step.uses?.startsWith("dtolnay/rust-toolchain@")));
  assert.ok(steps.some((step) => step.uses === "./.github/actions/setup-rust-sticky-cache"));

  const hydrate = findStep(steps, "Select and hydrate full fixture corpus");
  assert.equal(hydrate.run, "node tools/fixtures/davinci-dom-corpus-workflow.mjs hydrate");
  for (const pattern of [
    /git", \["ls-files", "--stage", "--", corpusRoot\]/,
    /expectedGitlinks = 146/,
    /artifactDir = "real-project-davinci-dom-corpus"/,
    /selected-gitlinks\.txt/,
    /"submodule",\s+"update",\s+"--init",\s+"--checkout",\s+"--depth",\s+"1",\s+"--jobs",\s+"8"/,
    /"submodule",\s+"update",\s+"--init",\s+"--checkout",\s+"--force"/,
    /"submodule", "status", "--", corpusRoot/,
  ]) {
    assert.match(helperSource, pattern);
  }

  const corpus = findStep(steps, "Run S2 DOM differential corpus");
  assert.equal(corpus.id, "davinci_dom_corpus");
  assert.equal(corpus["continue-on-error"], true);
  assert.equal(corpus.run, "node tools/fixtures/davinci-dom-corpus-workflow.mjs run");
  assert.match(helperSource, /VIZE_DAVINCI_DIFFERENTIAL_CORPUS: corpusRoot/);
  assert.match(helperSource, /"cargo",/);
  assert.match(helperSource, /"test",\s+"-p",\s+"vize_s1_to_s2"/);
  assert.match(helperSource, /"davinci-differential"/);
  assert.match(helperSource, /"davinci_dom_corpus"/);
  assert.match(helperSource, /dom-corpus\.log/);

  const finalize = findStep(steps, "Finalize S2 DOM corpus evidence");
  assert.equal(finalize.if, "${{ always() }}");
  assert.deepEqual(finalize.env, {
    VIZE_DAVINCI_DOM_CORPUS_OUTCOME: "${{ steps.davinci_dom_corpus.outcome }}",
  });
  assert.equal(finalize.run, "node tools/fixtures/davinci-dom-corpus-workflow.mjs finalize");
  assert.match(helperSource, /"record-only"/);
  assert.match(helperSource, /expectedDomOutputComparisons = 144/);
  assert.match(helperSource, /summary\.json/);
  assert.match(helperSource, /davinci-differential corpus scope\|davinci DOM corpus sweep/);
  assert.match(helperSource, /davinci DOM corpus old-lane error reasons/);
  assert.match(helperSource, /Davinci S2 DOM corpus failed/);

  const upload = findStep(steps, "Upload S2 DOM corpus evidence");
  assert.equal(upload.if, "${{ always() }}");
  assert.equal(upload.uses, "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a");
  assert.deepEqual(upload.with, {
    name: "real-project-davinci-dom-corpus",
    path: "real-project-davinci-dom-corpus",
    "if-no-files-found": "error",
    "retention-days": 30,
  });
});

test("S2 DOM corpus workflow helper extracts canonical evidence", () => {
  assert.deepEqual(
    parseFixtureGitlinks(
      [
        "100644 0123456789012345678901234567890123456789 0\tignored.txt",
        "160000 b6011381bc34a6b85ad669363513cb1a2eea6438 0\ttests/_fixtures/_git/airi",
        "160000 3ee62adffdcdfa4a37b2ed4e9c30636655d5fcd1 0\ttests/_fixtures/_git/create-vue",
      ].join("\n"),
    ),
    ["tests/_fixtures/_git/airi", "tests/_fixtures/_git/create-vue"],
  );
  assert.deepEqual(
    corpusEvidenceLines("\u001B[32mdavinci DOM corpus sweep: compared=1\u001B[0m\nx"),
    ["\u001B[32mdavinci DOM corpus sweep: compared=1\u001B[0m"],
  );
  assert.equal(verdictFor("failure", "record-only"), "success");
  assert.equal(verdictFor("cancelled", "record-only"), "cancelled");
  assert.equal(expectedGitlinks, 146);
  assert.equal(expectedDomOutputComparisons, 144);
});

test("S2 DOM corpus workflow extracts old-lane skip reasons from corpus logs", () => {
  const log = [
    "davinci-differential corpus scope: root=tests/_fixtures/_git scope=canonical closure_evidence=true submodules=146",
    "davinci DOM corpus sweep: files=3 unreadable=0 parsed=3 templates=3 compared=1 old_error_skips=2 s2_refusals=0 divergences=0",
    "corpus old-lane error skips (2):",
    '/repo/tests/_fixtures/_git/a.vue: 2 old-lane blocking errors: [CompilerError { code: InvalidEndTag, message: "Invalid end tag.", loc: None }, CompilerError { code: MissingEndTag, message: "Element is missing end tag.", loc: None }]',
    '/repo/tests/_fixtures/_git/b.vue: 1 old-lane blocking errors: [CompilerError { code: DuplicateAttribute, message: "Duplicate attribute.", loc: None }]',
    "",
    "corpus S2 refusals (0) by reason {}:",
  ].join("\n");

  assert.deepEqual(parseOldErrorReasons(log), {
    DuplicateAttribute: 1,
    InvalidEndTag: 1,
    MissingEndTag: 1,
  });
  assert.deepEqual(parseCorpusEvidence(log).oldErrorReasons, {
    DuplicateAttribute: 1,
    InvalidEndTag: 1,
    MissingEndTag: 1,
  });
});

test("S2 DOM corpus workflow prefers explicit old-lane reason counts", () => {
  const log = [
    "davinci DOM corpus sweep: files=3 unreadable=0 parsed=3 templates=3 compared=1 old_error_skips=2 s2_refusals=0 divergences=0",
    'davinci DOM corpus old-lane error reasons: {"InvalidEndTag": 2, "VIfSameKey": 1}',
  ].join("\n");

  assert.deepEqual(parseOldErrorReasons(log), {
    InvalidEndTag: 2,
    VIfSameKey: 1,
  });
  assert.deepEqual(parseCorpusEvidence(log).oldErrorReasons, {
    InvalidEndTag: 2,
    VIfSameKey: 1,
  });
});

test("S2 DOM corpus workflow gitlink constant matches the checkout index", () => {
  const indexed = parseFixtureGitlinks(
    execFileSync("git", ["ls-files", "--stage", "--", "tests/_fixtures/_git"], {
      encoding: "utf8",
    }),
  );

  assert.equal(indexed.length, expectedGitlinks);
});

test("S2 DOM corpus hydrate falls back from bulk shallow failures", () => {
  const artifact = mkdtempSync(join(tmpdir(), "vize-dom-corpus-"));
  const fixturePaths = Array.from(
    { length: expectedGitlinks },
    (_, index) => `tests/_fixtures/_git/project-${index}`,
  );
  const calls = [];
  const jobCount = (args) => {
    const index = args.indexOf("--jobs");
    return index === -1 ? undefined : args[index + 1];
  };
  const runCommand = (command, args) => {
    calls.push([command, args]);
    if (args[0] === "ls-files") {
      return fixturePaths
        .map((fixturePath, index) => `160000 ${String(index).padStart(40, "0")} 0\t${fixturePath}`)
        .join("\n");
    }
    if (args[0] === "submodule" && args[1] === "update" && args.includes("--jobs")) {
      if (args.includes("8")) throw new Error("bulk shallow failed");
      if (args.includes("tests/_fixtures/_git/project-7")) {
        throw new Error("single shallow failed");
      }
      return "";
    }
    if (args[0] === "submodule" && args[1] === "update" && args.includes("--force")) {
      return "";
    }
    if (args[0] === "submodule" && args[1] === "status") {
      return fixturePaths
        .map((fixturePath, index) => ` ${String(index).padStart(40, "0")} ${fixturePath}`)
        .join("\n");
    }
    throw new Error(`unexpected command: ${command} ${args.join(" ")}`);
  };
  try {
    assert.equal(hydrateCorpus({ artifact, runCommand }), 0);
    assert.ok(calls.some(([, args]) => jobCount(args) === "8"));
    assert.equal(calls.filter(([, args]) => jobCount(args) === "1").length, expectedGitlinks);
    assert.deepEqual(
      calls.filter(([, args]) => args.includes("--force")).map(([, args]) => args.at(-1)),
      ["tests/_fixtures/_git/project-7"],
    );
    assert.equal(
      readFileSync(join(artifact, "selected-gitlinks.txt"), "utf8").split(/\r?\n/).filter(Boolean)
        .length,
      expectedGitlinks,
    );
    assert.equal(
      readFileSync(join(artifact, "submodule-status.txt"), "utf8").split(/\r?\n/).filter(Boolean)
        .length,
      expectedGitlinks,
    );
  } finally {
    rmSync(artifact, { recursive: true, force: true });
  }
});

test("S2 DOM corpus workflow validates closure evidence artifacts", () => {
  const artifact = mkdtempSync(join(tmpdir(), "vize-dom-corpus-"));
  try {
    writeFileSync(
      join(artifact, "selected-gitlinks.txt"),
      Array.from({ length: 146 }, (_, index) => `tests/_fixtures/_git/project-${index}`)
        .join("\n")
        .concat("\n"),
    );
    writeFileSync(
      join(artifact, "submodule-status.txt"),
      Array.from(
        { length: 146 },
        (_, index) =>
          ` 0123456789abcdef0123456789abcdef01234567 tests/_fixtures/_git/project-${index}`,
      )
        .join("\n")
        .concat("\n"),
    );
    writeFileSync(
      join(artifact, "dom-corpus.log"),
      [
        "\u001B[32mdavinci-differential corpus scope: root=tests/_fixtures/_git scope=canonical closure_evidence=true submodules=146\u001B[0m",
        "davinci DOM corpus sweep: files=37448 unreadable=0 parsed=37448 templates=35000 compared=35000 old_error_skips=0 s2_refusals=0 divergences=0",
      ].join("\n"),
    );

    const validation = validateCorpusEvidence(artifact);
    assert.deepEqual(validation.failures, []);
    assert.equal(validation.manifestDomOutputComparisons, 144);
    assert.equal(validation.selectedGitlinks, 146);
    assert.equal(validation.submoduleStatusRows, 146);
    assert.deepEqual(parseCorpusEvidence(readFileSync(join(artifact, "dom-corpus.log"), "utf8")), {
      canonicalScope: true,
      closureEvidence: true,
      submodules: 146,
      files: 37448,
      unreadable: 0,
      parsed: 37448,
      templates: 35000,
      compared: 35000,
      oldErrorSkips: 0,
      oldErrorReasons: {},
      s2Refusals: 0,
      divergences: 0,
    });
  } finally {
    rmSync(artifact, { recursive: true, force: true });
  }
});

test("S2 DOM corpus workflow rejects stale or dirty evidence artifacts", () => {
  const artifact = mkdtempSync(join(tmpdir(), "vize-dom-corpus-"));
  try {
    writeFileSync(
      join(artifact, "selected-gitlinks.txt"),
      Array.from({ length: 142 }, (_, index) => `tests/_fixtures/_git/project-${index}`)
        .join("\n")
        .concat("\n"),
    );
    writeFileSync(join(artifact, "submodule-status.txt"), "");
    writeFileSync(
      join(artifact, "dom-corpus.log"),
      [
        "davinci-differential corpus scope: root=/tmp/tests/_fixtures/_git scope=smoke closure_evidence=false",
        "davinci DOM corpus sweep: files=1 unreadable=3 parsed=1 templates=1 compared=0 old_error_skips=2 s2_refusals=1 divergences=1",
        "corpus old-lane error skips (2):",
        '/repo/tests/_fixtures/_git/a.vue: 1 old-lane blocking errors: [CompilerError { code: InvalidEndTag, message: "Invalid end tag.", loc: None }]',
        '/repo/tests/_fixtures/_git/b.vue: 1 old-lane blocking errors: [CompilerError { code: VIfSameKey, message: "v-if/v-else-if branches must use unique keys.", loc: None }]',
        "",
        "corpus S2 refusals (1) by reason {}:",
      ].join("\n"),
    );

    assert.deepEqual(validateCorpusEvidence(artifact).failures, [
      "selected gitlinks 142 != 146",
      "submodule status rows 0 != 146",
      "corpus log is missing canonical closure evidence",
      "corpus log submodules 0 != 146",
      "corpus log proves no DOM-output comparisons",
      "corpus log skipped inputs: unreadable=3 old_error_skips=2 reasons=InvalidEndTag=1,VIfSameKey=1",
      "corpus log is not clean: s2_refusals=1 divergences=1",
    ]);
  } finally {
    rmSync(artifact, { recursive: true, force: true });
  }
});
