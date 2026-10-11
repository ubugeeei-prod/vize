import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const root = fileURLToPath(new URL("../../", import.meta.url));
const helper = "tools/support/compat/github/custom-directive-source-custody.py";
const recordedBefore = "67e3d0e6775cc3a1f29bfb9309d991252f9e75ff";
const recordedIntegrated = "0f39ac834eebdb02821acd4bd81ce8d2a3c48bd1";
const production = [
  "crates/vize_canon/src/virtual_ts/expressions/directive_values.rs",
  "crates/vize_canon/src/virtual_ts/expressions/directive_values/arguments.rs",
  "crates/vize_canon/src/virtual_ts/expressions/directive_values/presence.rs",
  "crates/vize_canon/src/virtual_ts/expressions/statements.rs",
].sort();
const compiled = (name) =>
  /^(?:crates|davinci|vendor|\.cargo)\//.test(name) ||
  ["Cargo.lock", "rust-toolchain.toml"].includes(name) ||
  name.endsWith("Cargo.toml");

// Offline real-Git boundary laws use exactly two explicit test-only identity
// substitutions. Actual pinned whole-body/native proof stays in full-history
// Actions; no authority predicate, source scope, or native oracle is replaced.
void test("integrated replay authenticates both complete source arms and refuses foreign bodies", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-directive-custody-"));
  const checkout = path.join(temporary, "source");
  const replaceBase = `refs/custody-law/${path.basename(temporary)}/`;
  const environment = {
    ...process.env,
    GIT_REPLACE_REF_BASE: replaceBase,
    PYTHONDONTWRITEBYTECODE: "1",
  };
  const git = (args, options = {}) =>
    execFileSync("git", args, {
      cwd: checkout,
      env: environment,
      maxBuffer: 16 * 1024 * 1024,
      ...options,
    });
  fs.mkdirSync(checkout);
  git(["init", "--quiet"]);
  git(["config", "user.name", "Custody law"]);
  git(["config", "user.email", "custody@example.invalid"]);
  git(["config", "commit.gpgsign", "false"]);
  const write = (name, body) => {
    const physical = path.join(checkout, name);
    fs.mkdirSync(path.dirname(physical), { recursive: true });
    fs.writeFileSync(physical, body);
  };
  const commit = (message) => {
    git(["add", "."]);
    git(["-c", "core.hooksPath=/dev/null", "commit", "--quiet", "-m", message]);
    return git(["rev-parse", "HEAD"]).toString().trim();
  };
  for (const name of production.filter((name) => !name.endsWith("/arguments.rs")))
    write(name, `historical fixture body: ${name}\n`);
  for (const name of [
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "crates/vize_canon/src/lib.rs",
  ])
    write(name, `original common fixture: ${name}\n`);
  const before = commit("test-only historical production");
  for (const name of production) write(name, `integrated fixture body: ${name}\n`);
  const integrated = commit("test-only integrated production");
  const driver = [
    ".github/workflows/typechecker-native-phases.yml",
    "crates/vize_canon/tests/native_custom_directive_arguments.rs",
    "docs/davinci/decisions/2026-09-27-level-restructure.md",
    "docs/davinci/decisions/2026-10-11-custom-directive-argument-values.md",
    "docs/davinci/decisions/2026-10-12-integrated-custom-directive-custody.md",
    "docs/davinci/plan/consumer-migration-surfaces/typechecker/vize_canon.tsv",
    "tools/support/compat/github/native-custom-directive-arguments.sh",
    "tests/tooling/custom-directive-source-custody.test.mjs",
    ...["App.vue.txt", "README.md", "cases.json", "original-custody.json", "tsconfig.json.txt"].map(
      (name) =>
        `tests/_fixtures/differential/typechecker/dynamic-custom-directive-argument/${name}`,
    ),
  ];
  for (const name of driver) write(name, `current driver fixture: ${name}\n`);
  const source = fs.readFileSync(path.join(root, helper), "utf8");
  const anchors = [
    [`historical_before = "${recordedBefore}"`, `historical_before = "${before}"`],
    [`integrated_after = "${recordedIntegrated}"`, `integrated_after = "${integrated}"`],
  ];
  let mapped = source;
  for (const [original, fixture] of anchors) {
    assert.equal(mapped.split(original).length, 2);
    mapped = mapped.replace(original, fixture);
  }
  assert.equal(
    anchors.reduce((body, [original, fixture]) => body.replace(fixture, original), mapped),
    source,
  );
  write(helper, mapped);
  // An accepted current-head source change outside the four historical paths
  // must appear, unchanged, in both counterfactual arms.
  write("crates/current-source.rs", "accepted incoming current source\n");
  write("Cargo.lock", "accepted incoming current lock\n");
  const head = commit("test-only current common source and mapped driver");
  try {
    const entries = git(["ls-tree", "-rz", "--full-tree", head])
      .toString()
      .split("\0")
      .filter(Boolean)
      .map((record) => {
        const [metadata, name] = record.split("\t");
        const [mode, kind, blob] = metadata.split(" ");
        return { name, mode, kind, blob };
      });
    const candidateBodies = new Map(
      production.map((name) => [name, fs.readFileSync(path.join(checkout, name))]),
    );
    const run = (phase = "candidate", baseline = before, candidate = head, options = {}) => {
      const output = path.join(temporary, `manifest-${phase}.json`);
      fs.rmSync(output, { force: true });
      const result = spawnSync(
        "python3",
        [...(options.optimize ? ["-O"] : []), "-B", helper, baseline, candidate, phase, output],
        {
          cwd: checkout,
          env: { ...environment, ...options.env },
          encoding: "utf8",
          maxBuffer: 1024 * 1024,
        },
      );
      return {
        ...result,
        packet: fs.existsSync(output) ? JSON.parse(fs.readFileSync(output, "utf8")) : null,
      };
    };
    const accept = (result) => {
      assert.equal(result.status, 0, result.stderr);
      assert.ok(result.packet);
      return result.packet;
    };
    const refuse = (result, message) => {
      assert.equal(result.status, 1, result.stderr);
      assert.equal(result.packet, null);
      assert.match(result.stderr, message);
    };
    const overlay = () => {
      for (const name of production) {
        const original = spawnSync("git", ["show", `${before}:${name}`], { cwd: checkout });
        if (original.status === 0) fs.writeFileSync(path.join(checkout, name), original.stdout);
        else {
          assert.equal(
            name,
            "crates/vize_canon/src/virtual_ts/expressions/directive_values/arguments.rs",
          );
          fs.rmSync(path.join(checkout, name));
        }
      }
    };
    const restore = () => {
      for (const [name, body] of candidateBodies) fs.writeFileSync(path.join(checkout, name), body);
    };
    const candidate = accept(run());
    assert.deepEqual(candidate.productionDelta, production);
    assert.deepEqual(
      candidate.compiledSourceManifest.map(({ path: name }) => name),
      entries
        .filter(({ name }) => compiled(name))
        .map(({ name }) => name)
        .sort(),
    );
    assert.equal(candidate.counterfactualAuthority.commonHead, head);
    assert.equal(candidate.counterfactualAuthority.integratedAfter, integrated);
    assert.equal(candidate.counterfactualAuthority.phase, "candidate");
    assert.ok(
      candidate.compiledSourceManifest.some(
        ({ path: name }) => name === "crates/vize_canon/tests/native_custom_directive_arguments.rs",
      ),
    );
    overlay();
    const baseline = accept(run("baseline"));
    assert.equal(baseline.counterfactualAuthority.phase, "baseline");
    assert.equal(baseline.counterfactualAuthority.commonHead, head);
    const withoutProduction = (packet) =>
      packet.compiledSourceManifest.filter(({ path: name }) => !production.includes(name));
    assert.deepEqual(withoutProduction(baseline), withoutProduction(candidate));
    assert.deepEqual(baseline.neutralDriverManifest, candidate.neutralDriverManifest);
    assert.deepEqual(
      baseline.counterfactualAuthority.historicalProduction,
      candidate.counterfactualAuthority.historicalProduction,
    );
    assert.equal(
      baseline.counterfactualAuthority.historicalProduction.filter(({ absent }) => absent).length,
      1,
    );
    for (const row of baseline.counterfactualAuthority.historicalProduction.filter(
      ({ absent }) => !absent,
    ))
      assert.deepEqual(
        baseline.compiledSourceManifest.find(({ path: name }) => name === row.path),
        row,
      );
    const absent = production.find((name) => name.endsWith("/arguments.rs"));
    const commonLock = path.join(checkout, "Cargo.lock");
    const originalLock = fs.readFileSync(commonLock);
    fs.appendFileSync(commonLock, "foreign baseline common lock\n");
    refuse(run("baseline"), /Cargo.lock/);
    fs.writeFileSync(commonLock, originalLock);
    fs.writeFileSync(path.join(checkout, absent), "// foreign historically absent body\n");
    refuse(run("baseline"), /historically absent production body exists/);
    restore();
    refuse(run("unknown"), /unknown counterfactual arm/);
    refuse(run("candidate", integrated), /unknown historical baseline/);
    refuse(run("candidate", before, integrated), /candidate must equal HEAD/);
    refuse(
      run("candidate", before, head, { optimize: true }),
      /optimized Python cannot authenticate source custody/,
    );
    refuse(
      run("candidate", before, head, { env: { PYTHONOPTIMIZE: "1" } }),
      /optimized Python cannot authenticate source custody/,
    );
    for (const name of [
      "Cargo.toml",
      "Cargo.lock",
      "crates/vize_canon/src/lib.rs",
      ...production,
    ]) {
      const physical = path.join(checkout, name);
      const original = fs.readFileSync(physical);
      fs.appendFileSync(physical, "\n// foreign source body\n");
      refuse(run(), new RegExp(name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")));
      fs.writeFileSync(physical, original);
      fs.rmSync(physical);
      refuse(run(), /source missing/);
      fs.writeFileSync(physical, original);
    }
    const foreign = path.join(checkout, "crates/custody-foreign.rs");
    fs.writeFileSync(foreign, "// unauthenticated compiled member\n");
    refuse(run(), /foreign compiled source/);
    fs.rmSync(foreign);
    const productionSource = path.join(checkout, production[0]);
    fs.chmodSync(productionSource, 0o755);
    refuse(run(), /source mode changed/);
    fs.chmodSync(productionSource, 0o644);
    const changedBlob = git(["hash-object", "-w", "--stdin"], { input: "// foreign production\n" })
      .toString()
      .trim();
    git(["update-index", "--cacheinfo", `100644,${changedBlob},${production[0]}`]);
    const changedTree = git(["write-tree"]).toString().trim();
    const changedHead = git(
      [
        "-c",
        "user.name=Custody law",
        "-c",
        "user.email=custody@example.invalid",
        "commit-tree",
        changedTree,
        "-p",
        head,
      ],
      { input: "test-only changed production\n" },
    )
      .toString()
      .trim();
    git(["update-ref", "HEAD", changedHead]);
    fs.writeFileSync(productionSource, "// foreign production\n");
    refuse(run("candidate", before, changedHead), /integrated production body changed/);
    git(["update-ref", "HEAD", head]);
    git(["read-tree", head]);
    restore();
    assert.deepEqual(accept(run()), candidate);
    // Private replacement namespace cannot affect another agent's Git reads.
    git(["update-ref", `${replaceBase}${integrated}`, changedHead]);
    assert.equal(
      git(["show", `${integrated}:${production[0]}`]).toString(),
      "// foreign production\n",
    );
    assert.deepEqual(accept(run()), candidate);
    fs.writeFileSync(productionSource, "// foreign production\n");
    refuse(run(), /source body changed/);
    restore();
    git(["update-ref", "-d", `${replaceBase}${integrated}`]);
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
