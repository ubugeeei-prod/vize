import assert from "node:assert/strict";
import {
  mkdtempSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import {
  compareResults,
  collectUnitEvidence,
  testResults,
} from "../../tools/support/compat/github/compare-unused-bindings-profiles.ts";

const detectorNames = [
  "a_producer_reading_outside_its_depends_is_refused_too",
  "an_undeclared_read_is_refused_exactly_even_when_computed",
];
const output = [
  ...detectorNames.map((name) => `test ${name} ... ok`),
  "test ignored_witness ... ignored, requires external fixture",
  "test result: ok. 2 passed; 0 failed; 1 ignored; 0 measured; 9 filtered out; finished in 1.23s",
  "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.00s",
].join("\n");

await test("profile comparison retains every selected status and every Cargo summary", () => {
  const before = testResults(output);
  const after = testResults(
    `${JSON.stringify({ reason: "compiler-artifact", fresh: true })}\n${output.replace("1.23s", "0.02s")}`,
  );
  compareResults(before, after, "fact_demand");
  assert.equal(before.cases.length, 3);
  assert.equal(before.summaries.length, 2);
  for (const changed of [
    output.replace("ignored_witness", "different_ignored_witness"),
    output.replace("9 filtered out", "8 filtered out"),
    output.replace("requires external fixture", "different reason"),
    output.replace(detectorNames[0], "different_case"),
  ])
    assert.throws(() => compareResults(before, testResults(changed), "fact_demand"));
});

await test("incomplete, failed and zero-test Cargo output cannot qualify", () => {
  for (const invalid of [
    "",
    output.replace(`test ${detectorNames[0]} ... ok\n`, ""),
    output.replace("test ignored_witness ... ignored, requires external fixture\n", ""),
    output.replace("2 passed; 0 failed", "1 passed; 1 failed"),
    output.replace("test result: ok", "test result: FAILED"),
    "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 99 filtered out; finished in 0.00s",
  ])
    assert.throws(() => testResults(invalid));
});

await test("equal output still requires both debug detectors and the actual corpus test", () => {
  const missing = testResults(output.replace(detectorNames[0], "release_builds_carry_no_detector"));
  assert.throws(() => compareResults(missing, missing, "fact_demand"));
  const ignored = testResults(
    output
      .replace(`test ${detectorNames[0]} ... ok`, `test ${detectorNames[0]} ... ignored`)
      .replace("2 passed; 0 failed; 1 ignored", "1 passed; 0 failed; 2 ignored"),
  );
  assert.throws(() => compareResults(ignored, ignored, "fact_demand"));
  const result = testResults(output);
  assert.throws(() => compareResults(result, result, "fact_spec"));
  const corpus = testResults(
    output.replace(detectorNames[0], "the_corpus_shard_agrees_with_the_specs"),
  );
  compareResults(corpus, corpus, "fact_spec");
});

await test("the controlled comparison is manual-only and retains the real seeded CLI recall", () => {
  const workflow = parse(
    readFileSync(
      new URL("../../.github/workflows/davinci-unused-bindings.yml", import.meta.url),
      "utf8",
    ),
  );
  assert.equal(workflow.on.workflow_dispatch.inputs["compare-profiles"].default, false);
  const steps = workflow.jobs["unused-bindings"].steps;
  const compare = steps.find((step) => step.run?.includes("compare-unused-bindings-profiles.ts"));
  assert.equal(
    compare.if,
    "${{ github.event_name == 'workflow_dispatch' && inputs.compare-profiles }}",
  );
  assert.equal(
    compare.env.VIZE_DAVINCI_FACT_CORPUS,
    "tests/_fixtures/_git/splitpanes,tests/_fixtures/_git/layoutit-grid,tests/_fixtures/_git/cssgridgenerator",
  );
  assert.ok(
    steps.some((step) =>
      step.run?.includes(
        "cargo build --profile ci --message-format=json-render-diagnostics -p vize | tee",
      ),
    ),
  );
  const recall = steps.find((step) => step.run?.includes("davinci-unused-setup-bindings.test.ts"));
  assert.equal(recall.if, undefined);
  assert.match(recall.run, /target\/ci\/vize.*davinci-fpfn-pilots\.test\.ts/);
  assert.match(steps.at(-1).if, /always\(\)/);
});

await test("unit evidence retains actual artifact identity and only emitted fingerprint JSON", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-unit-evidence-"));
  try {
    const target = join(root, "target"),
      output = join(target, "unused-bindings-units");
    mkdirSync(output, { recursive: true });
    const packets = [];
    const hashes = ["1111111111111111", "2222222222222222"];
    const fixtures = [
      {
        profile: "ci",
        kind: "proc-macro",
        test: false,
        hash: hashes[0],
        flags: ["--cfg", "host_unit"],
      },
      {
        profile: "aarch64-unknown-linux-gnu/ci",
        kind: "lib",
        test: false,
        hash: hashes[0],
        flags: ["--cfg", "cross_unit"],
      },
      { profile: "ci", kind: "test", test: true, hash: hashes[1], flags: ["--cfg", "test_unit"] },
      {
        profile: "release",
        kind: "lib",
        test: false,
        hash: hashes[1],
        flags: ["--cfg", "release_unit"],
      },
      {
        profile: "ci",
        kind: "custom-build",
        test: false,
        hash: "4444444444444444",
        flags: ["--cfg", "build_unit"],
      },
    ];
    for (const { profile, kind, test, hash, flags } of fixtures) {
      const filename =
        kind === "custom-build"
          ? join(target, profile, "build", `package_name-${hash}`, "build-script-build")
          : join(
              target,
              profile,
              "deps",
              `${test ? "" : "lib"}same_name-${hash}${test ? "" : kind === "proc-macro" ? ".so" : ".rlib"}`,
            );
      mkdirSync(dirname(filename), { recursive: true });
      writeFileSync(filename, "compiled output must not be copied");
      const fp = join(
        target,
        profile,
        ".fingerprint",
        `package_name-${hash}`,
        "lib-same_name.json",
      );
      mkdirSync(dirname(fp), { recursive: true });
      writeFileSync(fp, JSON.stringify({ rustflags: flags, compile_kind: profile }));
      packets.push({
        reason: "compiler-artifact",
        package_id: "registry+https://example.test#package_name@1.0.0",
        target: { name: "same_name", kind: [kind] },
        profile: { test, opt_level: profile === "release" ? "3" : "0" },
        features: [profile],
        fresh: test,
        filenames: [filename],
      });
    }
    const decoy = join(target, "ci/.fingerprint/unobserved-3333333333333333/lib-decoy.json");
    mkdirSync(dirname(decoy), { recursive: true });
    writeFileSync(decoy, "{}");
    const script = {
      reason: "build-script-executed",
      package_id: "same",
      cfgs: ["native_unit"],
      env: [["VISIBLE_CFG", "1"]],
    };
    writeFileSync(
      join(output, "ci-example.jsonl"),
      [...packets, script].map((packet) => JSON.stringify(packet)).join("\n") +
        '\ntest actual_case ... ok\n{"reason":"compiler-artifact"\n',
    );
    const result = collectUnitEvidence(output, target, { source: "fixture" });
    assert.deepEqual(
      result.units.map((unit) => unit.packet),
      packets,
    );
    assert.deepEqual(result.buildScripts[0].packet, script);
    assert.equal(result.parseGaps.length, 1);
    assert.equal(result.units.length, fixtures.length);
    const retained = result.units.flatMap((unit) => unit.fingerprints.map((fp) => fp.path));
    assert.equal(new Set(retained).size, fixtures.length);
    // upload-artifact excludes hidden components by default. Retain the exact
    // Cargo source path in metadata, while exporting through visible paths.
    assert.ok(retained.every((name) => name.split(/[\\/]/).every((part) => !part.startsWith("."))));
    assert.ok(
      result.units.every((unit) =>
        unit.fingerprints.every((fp) => fp.source.includes(".fingerprint")),
      ),
    );
    for (const [index, name] of retained.entries())
      assert.deepEqual(
        JSON.parse(readFileSync(join(output, name), "utf8")).rustflags,
        fixtures[index].flags,
      );
    assert.ok(
      readdirSync(output, { recursive: true }).every(
        (name) => !String(name).includes("unobserved") && !String(name).endsWith(".rlib"),
      ),
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

await test("unhashed, missing and escaped unit evidence stays explicitly unavailable", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-unit-evidence-gaps-"));
  try {
    const target = join(root, "target"),
      output = join(target, "unused-bindings-units");
    mkdirSync(output, { recursive: true });
    const executable = join(target, "ci/vize"),
      missing = join(target, "ci/deps/libmissing-1111111111111111.rlib");
    mkdirSync(dirname(missing), { recursive: true });
    writeFileSync(executable, "");
    writeFileSync(missing, "");
    const secret = join(root, "secret-2222222222222222.rlib");
    writeFileSync(secret, "must not be copied");
    const escape = join(target, "ci/deps/libescape-2222222222222222.rlib");
    symlinkSync(secret, escape);
    writeFileSync(
      join(output, "partial.jsonl"),
      [executable, missing, escape]
        .map((filename) => JSON.stringify({ reason: "compiler-artifact", filenames: [filename] }))
        .join("\n"),
    );
    const result = collectUnitEvidence(output, target);
    assert.equal(result.units.length, 3);
    assert.equal(result.units.flatMap((unit) => unit.fingerprints).length, 0);
    assert.deepEqual(
      result.units.map((unit) => unit.gaps[0].reason),
      [
        "no emitted unit hash; name matching refused",
        "fingerprint root unavailable",
        "output missing or outside target",
      ],
    );
    assert.match(readFileSync(join(output, "units.json"), "utf8"), /unavailable|missing/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

await test("tee preserves Cargo success/failure and partial raw output without invoking real Cargo", () => {
  const workflow = parse(
    readFileSync(
      new URL("../../.github/workflows/davinci-unused-bindings.yml", import.meta.url),
      "utf8",
    ),
  );
  const steps = workflow.jobs["unused-bindings"].steps;
  const root = mkdtempSync(join(tmpdir(), "vize-unit-pipefail-"));
  try {
    mkdirSync(join(root, "target/unused-bindings-units"), { recursive: true });
    for (const status of [17, 0])
      for (const name of ["Check independent specs and real rule dispatch", "Build the real CLI"]) {
        const script = steps.find((step) => step.name === name).run;
        const result = spawnSync(
          "bash",
          [
            "-e",
            "-c",
            `cargo() { printf '%s\\n' '{"reason":"compiler-artifact","filenames":[]}'; return ${status}; };\n${script}`,
          ],
          { cwd: root, encoding: "utf8" },
        );
        assert.equal(result.status, status, result.stderr);
        if (name.startsWith("Check"))
          assert.equal(
            readdirSync(join(root, "target/unused-bindings-units")).filter((entry) =>
              entry.startsWith("ci-fact"),
            ).length,
            status === 0 ? 2 : 1,
          );
      }
    assert.ok(
      readFileSync(
        join(root, "target/unused-bindings-units/ci-fact_demand.jsonl"),
        "utf8",
      ).includes("compiler-artifact"),
    );
    assert.ok(
      readFileSync(join(root, "target/unused-bindings-units/ci-vize.jsonl"), "utf8").includes(
        "compiler-artifact",
      ),
    );
    for (const name of [
      "Collect only emitted Cargo unit fingerprints",
      "Preserve Cargo unit evidence even on failure",
    ])
      assert.equal(steps.find((step) => step.name === name).if, "${{ always() }}");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
