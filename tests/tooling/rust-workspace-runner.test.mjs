import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

const runner = new URL(
  "../../tools/support/compat/github/run-rust-workspace-tests.mjs",
  import.meta.url,
);

function fixture(
  mode,
  failure = "",
  version = "0.9.146",
  profile = "ci",
  extra = [],
  configuration = {},
) {
  const cwd = mkdtempSync(join(tmpdir(), "vize-workspace-runner-"));
  const calls = join(cwd, "calls.jsonl");
  const receipt = join(cwd, "receipt.json");
  const cargo = join(cwd, "cargo");
  mkdirSync(join(cwd, ".cargo"));
  mkdirSync(join(cwd, ".config"));
  for (const [file, content] of Object.entries({
    "Cargo.toml":
      '[workspace]\nmembers = []\n[profile.ci]\ninherits = "dev"\ndebug = false\nincremental = false\ncodegen-units = 256\n' +
      (configuration.manifestSuffix ?? ""),
    "Cargo.lock": "version = 4\n",
    "rust-toolchain.toml": '[toolchain]\nchannel = "1.99.1"\n',
    ".cargo/config.toml": "# original linker configuration\n",
    ".config/nextest.toml": "[profile.full]\nfail-fast = false\nretries = 0\n",
  }))
    writeFileSync(join(cwd, file), content);
  const git = (...args) => execFileSync("git", args, { cwd, stdio: "ignore" });
  git("init", "-q");
  git("add", ".");
  git("-c", "user.name=CI Test", "-c", "user.email=ci@example.invalid", "commit", "-qm", "source");
  writeFileSync(
    join(cwd, "rustc"),
    `#!${process.execPath}\nconsole.log('rustc ${configuration.compiler ?? "1.99.1"} (fixture)\\nhost: fixture');\n`,
    { mode: 0o755 },
  );
  writeFileSync(
    cargo,
    `#!${process.execPath}\nconst fs = require('node:fs'); const args = process.argv.slice(2); if (args.join(' ') === 'nextest --version') { console.log('cargo-nextest ${version}'); } else if (args.join(' ') === '-Vv') { console.log('cargo 1.99.1 (fixture)'); } else { fs.appendFileSync(process.env.CALLS, JSON.stringify(args)+'\\n'); if (args.includes(process.env.FAIL)) process.exit(37); if (args.includes('list')) console.log(JSON.stringify({'rust-suites':{fixture:{'binary-id':'fixture',status:'listed','package-name':'fixture','binary-name':'fixture',testcases:{case:{ignored:false,'filter-match':{status:'matches'}}}}}})); }\n`,
    { mode: 0o755 },
  );
  const result = spawnSync(
    process.execPath,
    [runner.pathname, "--runner", mode, "--cargo-profile", profile, "--output", receipt, ...extra],
    {
      cwd,
      encoding: "utf8",
      env: {
        ...process.env,
        GITHUB_SHA: "",
        PATH: `${cwd}:${process.env.PATH}`,
        CALLS: calls,
        FAIL: failure,
        VIZE_TEST_REQUIRE_TSGO: "1",
        VIZE_NUXT_CONFIG_ITERATIONS: "100",
        ...configuration.env,
      },
    },
  );
  const observed = {
    result,
    calls: (() => {
      try {
        return readFileSync(calls, "utf8").trim().split("\n").map(JSON.parse);
      } catch {
        return [];
      }
    })(),
    receipt: (() => {
      try {
        return JSON.parse(readFileSync(receipt, "utf8"));
      } catch {
        return null;
      }
    })(),
  };
  rmSync(cwd, { recursive: true, force: true });
  return observed;
}

void test("full nextest keeps workspace selection, pinned version, and explicit doctests", () => {
  const f = fixture("nextest");
  assert.equal(f.result.status, 0, f.result.stderr);
  assert.deepEqual(f.calls, [
    [
      "nextest",
      "list",
      "--locked",
      "--workspace",
      "--cargo-profile",
      "ci",
      "--profile",
      "full",
      "--run-ignored",
      "all",
      "--message-format",
      "json",
    ],
    ["nextest", "run", "--locked", "--workspace", "--cargo-profile", "ci", "--profile", "full"],
    ["test", "--locked", "--workspace", "--profile", "ci", "--doc"],
  ]);
  assert.equal(f.receipt.runner, "nextest");
  assert.equal(f.receipt.status, "passed");
  assert.equal(f.receipt.cargoProfile, "ci");
  assert.deepEqual(f.receipt.source.inferredProfileDefaults, {
    basis: "committed-manifest-config-and-runtime-environment",
    debugAssertions: true,
    overflowChecks: true,
  });
  assert.equal(f.receipt.phases.length, 3);
});

void test("serial reference invokes the original complete cargo workspace recipe", () => {
  const f = fixture("cargo", "", "0.9.146", "test");
  assert.equal(f.result.status, 0, f.result.stderr);
  assert.deepEqual(f.calls, [["test", "--locked", "--workspace", "--profile", "test"]]);
});

void test("failure in either ordinary tests or doctests fails the complete receipt", () => {
  for (const phase of ["run", "--doc"]) {
    const f = fixture("nextest", phase);
    assert.equal(f.result.status, 37, f.result.stderr);
    assert.equal(f.receipt.status, "failed");
    assert.equal(f.calls.length, 3, "doctests remain attempted after an ordinary-test failure");
    assert.equal(f.receipt.phases.find((entry) => entry.exitCode === 37).status, "failed");
  }
});

void test("inventory failure does not run a partial workspace and still attempts doctests", () => {
  const f = fixture("nextest", "list");
  assert.equal(f.result.status, 37, f.result.stderr);
  assert.equal(f.calls.length, 2);
  assert.ok(f.calls.at(-1).includes("--doc"));
});

void test("old-profile inventory is opt-in and cannot masquerade as a full test run", () => {
  const f = fixture("nextest", "", "0.9.146", "test", ["--inventory-only"]);
  assert.equal(f.result.status, 0, f.result.stderr);
  assert.equal(f.calls.length, 1);
  assert.ok(f.calls[0].includes("test"));
  assert.equal(f.receipt.inventoryOnly, true);
  assert.deepEqual(
    f.receipt.phases.map((phase) => phase.name),
    ["inventory"],
  );
});

void test("unknown runner or foreign nextest fails before executing test commands", () => {
  for (const [mode, version] of [
    ["unknown", "0.9.146"],
    ["nextest", "0.9.145"],
  ]) {
    const f = fixture(mode, "", version);
    assert.notEqual(f.result.status, 0);
    assert.deepEqual(f.calls, []);
  }
});

void test("source/compiler/profile/runtime mismatches fail before any workspace test command", () => {
  for (const configuration of [
    { compiler: "1.99.0" },
    { env: { GITHUB_SHA: "f".repeat(40) } },
    { env: { VIZE_TEST_REQUIRE_TSGO: "0" } },
    { env: { VIZE_TEST_DISABLE_TSGO: "1" } },
    { env: { VIZE_NUXT_CONFIG_ITERATIONS: "1" } },
    { env: { CARGO_PROFILE_CI_DEBUG_ASSERTIONS: "false" } },
    { env: { RUSTFLAGS: "-Cdebug-assertions=off" } },
    { manifestSuffix: "debug-assertions = false\n" },
    { manifestSuffix: "[profile.test]\noverflow-checks = false\n" },
  ]) {
    const f = fixture("nextest", "", "0.9.146", "ci", [], configuration);
    assert.notEqual(f.result.status, 0, JSON.stringify(configuration));
    assert.deepEqual(f.calls, []);
  }
});
