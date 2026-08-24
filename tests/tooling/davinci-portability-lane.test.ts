import assert from "node:assert/strict";
import { test } from "node:test";

import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

// TS-24 (davinci-road/plan/test-suites.md): the wasm32-wasip2 portability
// lanes for the four Davinci `no_std` stage libraries. The lanes ride `clippy-and-test`
// as steps rather than as their own job because `.github/workflows/check.yml`
// is over the 350-line ratchet and must not grow
// (davinci-road/plan/phase-2-records/p2-14.md); this file is what keeps that
// placement from silently dissolving. S0 (`vize_s0`, package `vize_carton`)
// remains the approved `std` foundation recorded in no-std-boundary.md.

const portableStageCrates = [
  "vize_davinci",
  "vize_sinopia",
  "vize_disegno",
  "vize_ricalco",
] as const;

const packageArgs = portableStageCrates.map((crate) => `-p ${crate}`).join(" ");
const defaultLane = `cargo check ${packageArgs} --lib --target wasm32-wasip2`;

test("TS-24: the wasm32-wasip2 lanes ride the required clippy-and-test job", () => {
  const workflow = readRepoFile(".github", "workflows", "check.yml");
  const job = workflowJobBody(workflow, "clippy-and-test");

  // Required on every pull request: clippy-and-test carries no event guard,
  // and it sits in the `needs:` list of test-report, the required status
  // check (tests/tooling/github-workflows-check-gate.test.ts deep-equals that
  // list, so membership cannot drift without failing there too).
  assert.doesNotMatch(
    job,
    /^ {4}if:/m,
    "clippy-and-test must stay unconditional: TS-24 is required for the core crates",
  );
  assert.match(workflowJobBody(workflow, "test-report"), /- clippy-and-test\b/);

  // The wasip2 std installs through the pinned toolchain: cargo runs under
  // rust-toolchain.toml (channel 1.95.0), so a `targets:` input on the
  // stable action never reaches the build - the first CI run proved that.
  // rustup materializes the toolchain file's targets, so the pin lives there.
  const toolchainFile = readRepoFile("rust-toolchain.toml");
  assert.match(toolchainFile, /^targets = \[.*"wasm32-wasip2".*\]$/m);

  // Both checks target libraries only. The host `davinci-opt` binary must not
  // become evidence for a `no_std` claim merely because WASI provides std.
  assert.ok(
    job.includes(`        run: ${defaultLane} && ${defaultLane} --no-default-features`),
    "TS-24 must check all four stage libraries with and without default features",
  );
  assert.doesNotMatch(job, /cargo check[^\n]*-p (?:vize_s0|vize_carton)[^\n]*wasm32-wasip2/);
});

test("the no_std claim stays on all four stage libraries and excludes the std S0 foundation", () => {
  // The lane's target carries std (wasm32-wasip2 is not a std-less build), so
  // the `no_std` half of the claim is held by these attributes; the build
  // proves they are honest (a `std::` path in either crate stops compiling).
  // A library joining the claim must appear here, in both lane commands above,
  // and in no-std-boundary.md's ledger.
  for (const crate of portableStageCrates) {
    const lib = readRepoFile("crates", crate, "src", "lib.rs");
    assert.match(lib, /^#!\[no_std\]$/m, `${crate} must keep #![no_std]`);
    assert.match(lib, /^extern crate alloc;$/m, `${crate} must keep extern crate alloc`);
  }

  const workspace = readRepoFile("Cargo.toml");
  assert.match(
    workspace,
    /^vize_s0 = \{ package = "vize_carton", path = "crates\/vize_carton", version = "=[^"]+" \}$/m,
  );
  const carton = readRepoFile("crates", "vize_carton", "src", "lib.rs");
  assert.doesNotMatch(carton, /^#!\[no_std\]$/m, "S0 is the accepted std host foundation");

  const davinciManifest = readRepoFile("crates", "vize_davinci", "Cargo.toml");
  assert.match(davinciManifest, /^path = "src\/bin\/davinci-opt\/main\.rs"$/m);
});
