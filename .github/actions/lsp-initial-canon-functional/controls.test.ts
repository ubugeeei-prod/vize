import assert from "node:assert/strict";
import path from "node:path";
import test from "node:test";
import { pathToFileURL } from "node:url";
import { SDK } from "./source-pins.ts";
import {
  TESTS,
  defaultCliArtifact,
  admittedFixture,
  HOVER,
  checkAmbient,
  checkParent,
  checkExecution,
  checkPackets,
  checkNativeExit,
  childEnv,
  libraryArtifact,
  sdkArtifacts,
  type Obj,
} from "./laws.ts";

const native = "/provider/tsgo";
const fixture = {
  source: "source bytes\n",
  shared: "shared bytes\n",
  expected: { kind: "full", items: [{ code: 2322, unknown: { retained: true } }] },
  editedExpected: { kind: "full", items: [] },
  config: { compilerOptions: { strict: true } },
};
const result = (fqn: string): string =>
  `running 1 test\ntest ${fqn} ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 301 filtered out; finished in 0.1s\n`;
const marker = (name: string, rows: unknown[]): string =>
  rows.map((row) => `${name} ${JSON.stringify(row)}\n`).join("");

await test("workflow source has exactly the reviewed direct parent", () => {
  const candidate = "a".repeat(40);
  checkParent(candidate, candidate);
  for (const raw of ["", `${candidate} ${"b".repeat(40)}`, "b".repeat(40)])
    assert.throws(() => checkParent(raw, candidate));
});
function factory(): Obj[] {
  const packet = {
    initial: { native: true, unknown: { whole: [1, 2] } },
    bad: fixture.expected,
    repaired: fixture.editedExpected,
    beforeEmpty: { opaque: [1, { all: "kept" }] },
    afterEmpty: { opaque: [1, { all: "kept" }] },
    canonSource: [...Buffer.from("source bytes\n")],
    canonShared: [...Buffer.from("shared bytes\n")],
    canonConfig: [...Buffer.from(JSON.stringify(fixture.config))],
  };
  return [false, true].map((direct, index) => ({
    direct,
    native,
    packet: structuredClone(packet),
    owner: { pid: index + 10, birth: index + 20, executable: native },
    authoredRoot: "/authored",
    canonRoot: "/canon",
    config: "/canon/tsconfig.selected.json",
  }));
}
function operations(): Obj[] {
  return ["none", "metadata", "otherOpen", "otherConfig"].map((operation) => ({
    operation,
    native,
    initialEpoch: 1,
    beforeConsume: operation === "none" ? 1 : 2,
    afterConsume: operation === "none" ? 2 : 5,
    documentUri: "file:///canon/App.vue.ts",
    supportUri: "file:///canon/support.ts",
    actual: structuredClone(HOVER),
    expected: structuredClone(HOVER),
  }));
}

await test("both exact FQNs require one real passed test and one summary", () => {
  for (const selected of TESTS) {
    checkExecution(result(selected.fqn), selected.fqn);
    for (const mutant of [
      "",
      result(selected.fqn).replace("1 passed", "0 passed"),
      result(selected.fqn).replace("0 ignored", "1 ignored"),
      result(selected.fqn).replace("0 failed", "1 failed"),
      result(selected.fqn) + result(selected.fqn),
      result("foreign"),
      result(selected.fqn).replace("0 measured", "1 measured"),
    ])
      assert.throws(() => checkExecution(mutant, selected.fqn));
  }
});
await test("a disabled-body fake pass cannot qualify without whole native markers", () => {
  for (const selected of TESTS) {
    checkExecution(result(selected.fqn), selected.fqn);
    assert.throws(() => checkPackets(selected.label, result(selected.fqn), native, fixture));
  }
});
await test("native exit signal error stderr and extra fields stay refusals", () => {
  const good = { status: 0, signal: null, error: null };
  checkNativeExit(good, "");
  for (const value of [
    { ...good, status: null },
    { ...good, status: 101 },
    { ...good, signal: "SIGTERM" },
    { ...good, error: "spawn" },
    { ...good, unknown: true },
  ])
    assert.throws(() => checkNativeExit(value, ""));
  assert.throws(() => checkNativeExit(good, "warning\n"));
});
await test("DISABLE even empty is refused and REQUIRE is exactly one", () => {
  assert.deepEqual(childEnv({}, native), { CORSA_PATH: native, VIZE_TEST_REQUIRE_TSGO: "1" });
  for (const value of ["", "0", "1"])
    assert.throws(() => childEnv({ VIZE_TEST_DISABLE_TSGO: value }, native));
  const good = { CARGO_INCREMENTAL: "0", CARGO_BUILD_JOBS: "16", RAYON_NUM_THREADS: "8" };
  checkAmbient(good);
  for (const name of [
    "CORSA_PATH",
    "RUSTC_WRAPPER",
    "CARGO_TARGET_DIR",
    "VIZE_TEST_DISABLE_TSGO",
    "CARGO_PROFILE_RELEASE_OPT_LEVEL",
  ])
    assert.throws(() => checkAmbient({ ...good, [name]: "" }));
});
await test("old and direct factory packets retain every complete nested field", () => {
  const rows = factory();
  assert.deepEqual(checkPackets("factory", marker(TESTS[0].marker, rows), native, fixture), rows);
  const mutated = factory();
  (mutated[1]!.packet as Obj).initial = { native: true, unknown: { whole: [1, 3] } };
  assert.throws(() => checkPackets("factory", marker(TESTS[0].marker, mutated), native, fixture));
});
await test("factory missing duplicate reordered foreign and unknown packets refuse", () => {
  const rows = factory();
  for (const list of [
    rows.slice(0, 1),
    [...rows, rows[0]],
    [...rows].reverse(),
    [rows[0], { ...rows[1], native: "/wrong" }],
    [rows[0], { ...rows[1], extra: true }],
  ])
    assert.throws(() => checkPackets("factory", marker(TESTS[0].marker, list), native, fixture));
  assert.throws(() =>
    checkPackets("factory", marker("INITIAL_CANON_UNKNOWN", rows), native, fixture),
  );
  const broken = factory();
  (broken[1]!.packet as Obj).repaired = fixture.expected;
  assert.throws(() => checkPackets("factory", marker(TESTS[0].marker, broken), native, fixture));
});
await test("all four mutation roles require whole authored native hover and epoch law", () => {
  const rows = operations();
  assert.deepEqual(
    checkPackets("operations", marker(TESTS[1].marker, rows), native, fixture),
    rows,
  );
  for (const change of [
    { actual: null },
    { actual: { ...HOVER, unknown: true } },
    { afterConsume: 3 },
    { supportUri: "http://foreign" },
  ]) {
    const mutated = operations();
    mutated[1] = { ...mutated[1], ...change };
    assert.throws(() =>
      checkPackets("operations", marker(TESTS[1].marker, mutated), native, fixture),
    );
  }
  assert.throws(() =>
    checkPackets("operations", marker(TESTS[1].marker, [...rows].reverse()), native, fixture),
  );
});

function artifacts(repo: string, target: string): Obj[] {
  return [
    ...Object.entries(SDK).map(([name, sdk]) => ({
      reason: "compiler-artifact",
      package_id: `${sdk.source}#${name}@${sdk.version}`,
      target: { name, kind: ["lib"] },
      features: [],
    })),
    {
      reason: "compiler-artifact",
      package_id: `path+${pathToFileURL(path.join(repo, "crates/vize_canon")).href}#vize_canon@1`,
      target: {
        name: "vize_canon",
        kind: ["lib"],
        src_path: path.join(repo, "crates/vize_canon/src/lib.rs"),
      },
      manifest_path: path.join(repo, "crates/vize_canon/Cargo.toml"),
      features: ["native"],
      profile: { test: true },
      executable: path.join(target, "ci/deps/vize_canon-123"),
    },
    { reason: "build-finished", success: true },
  ];
}
await test("actual library context must be Canon native ci and all seven registry SDK4 roles", () => {
  const repo = "/repo";
  const target = "/fresh-target";
  const rows = artifacts(repo, target);
  assert.equal(
    libraryArtifact(rows, target, repo).executable,
    "/fresh-target/ci/deps/vize_canon-123",
  );
  assert.equal(sdkArtifacts(rows).length, 7);
  for (const mutant of [
    rows.slice(1),
    rows.map((row) => ({ ...row, package_id: String(row.package_id).replace("beta.4", "beta.3") })),
    rows.map((row) => ({ ...row, features: ["profiling"] })),
  ])
    assert.throws(() => libraryArtifact(mutant, target, repo));
  const wrong = artifacts(repo, target);
  wrong[7] = { ...wrong[7], executable: "/foreign/ci/deps/vize_canon-123" };
  assert.throws(() => libraryArtifact(wrong, target, repo));
});

// Complete compile-consumed source fixture, not a normalized projection.
const actualFixtureBytes = Buffer.from(String.raw`{
  "description": "Retained original diagnosing native process; whole authored answers, no Program-identity claim.",
  "source": "import { value } from './shared.js';\nexport const answer: string = value;\n",
  "leafEdit": "import { value } from './shared.js';\nexport const answer: number = value;\n",
  "shared": "export const value = 1;\n",
  "sharedEdit": "export const value = 'stable';\n",
  "config": {
    "compilerOptions": {
      "noEmit": true,
      "strict": true,
      "types": [],
      "target": "ESNext",
      "module": "ESNext",
      "moduleResolution": "Bundler",
      "moduleDetection": "force",
      "skipLibCheck": true,
      "allowJs": true,
      "checkJs": true
    },
    "include": ["**/*"]
  },
  "expected": {
    "kind": "full",
    "items": [
      {
        "range": {
          "start": {
            "line": 1,
            "character": 13
          },
          "end": {
            "line": 1,
            "character": 19
          }
        },
        "severity": 1,
        "code": 2322,
        "source": "ts",
        "message": "Type 'number' is not assignable to type 'string'."
      }
    ]
  },
  "editedExpected": {
    "kind": "full",
    "items": []
  },
  "authored": "answer",
  "originalRefusal": {
    "projection": "IncompleteFile",
    "issues": [
      {
        "unit": 17,
        "start": 50,
        "end": 72,
        "kind": "UnsupportedSyntax"
      }
    ]
  },
  "admitted": {
    "description": "Genuine admitted primitive-return Module retains whole imported-type answer and physical native process.",
    "source": "import { value } from './shared.js';\nexport function answer(): string { return value; }\n",
    "leafEdit": "import { value } from './shared.js';\nexport function answer(): number { return value; }\n",
    "shared": "export const value = 1;\n",
    "sharedEdit": "export const value = 'stable';\n",
    "config": {
      "compilerOptions": {
        "noEmit": true,
        "strict": true,
        "types": [],
        "target": "ESNext",
        "module": "ESNext",
        "moduleResolution": "Bundler",
        "moduleDetection": "force",
        "skipLibCheck": true,
        "allowJs": true,
        "checkJs": true
      },
      "include": ["**/*"]
    },
    "expected": {
      "kind": "full",
      "items": [
        {
          "range": {
            "start": {
              "line": 1,
              "character": 35
            },
            "end": {
              "line": 1,
              "character": 41
            }
          },
          "severity": 1,
          "code": 2322,
          "source": "ts",
          "message": "Type 'number' is not assignable to type 'string'."
        }
      ]
    },
    "editedExpected": {
      "kind": "full",
      "items": []
    },
    "authored": "return"
  }
}
`);
const wrongRetirementFixtureBytes = Buffer.from(String.raw`{
  "description": "Native API project replacement reaps its previous process before spawning while the editor retains the complete diagnostic answer.",
  "source": "import { value } from './shared.js';\nexport function answer(): string { return value; }\n",
  "leafEdit": "import { value } from './shared.js';\nexport function answer(): number { return value; }\n",
  "shared": "export const value = 1;\n",
  "sharedEdit": "export const value = 'stable';\n",
  "config": {
    "compilerOptions": {
      "noEmit": true,
      "strict": true,
      "types": [],
      "target": "ESNext",
      "module": "ESNext",
      "moduleResolution": "Bundler",
      "moduleDetection": "force",
      "skipLibCheck": true,
      "allowJs": true,
      "checkJs": true
    },
    "include": ["**/*"]
  },
  "expected": {
    "kind": "full",
    "items": [
      {
        "range": {
          "start": {
            "line": 1,
            "character": 35
          },
          "end": {
            "line": 1,
            "character": 41
          }
        },
        "severity": 1,
        "code": 2322,
        "source": "ts",
        "message": "Type 'number' is not assignable to type 'string'."
      }
    ]
  },
  "editedExpected": {
    "kind": "full",
    "items": []
  },
  "authored": "return",
  "transitions": [
    "same-root-topology-reload",
    "same-root-topology-reload",
    "materialized-project-mode"
  ]
}
`);
await test("only the exact compile-consumed 7698 admitted fixture is accepted", () => {
  const envelope = JSON.parse(actualFixtureBytes.toString("utf8"));
  assert.deepEqual(admittedFixture(actualFixtureBytes), envelope.admitted);
  assert.throws(() => admittedFixture(wrongRetirementFixtureBytes));
  const changed = structuredClone(envelope);
  changed.admitted.source += "// changed admitted source\n";
  assert.notEqual(changed.admitted.source, envelope.admitted.source);
  assert.throws(() => admittedFixture(Buffer.from(JSON.stringify(changed))));
});
await test("equal wrong source or shared bytes in both native rows still refuse", () => {
  for (const field of ["canonSource", "canonShared"]) {
    const rows = factory();
    for (const row of rows) (row.packet as Obj)[field] = [...Buffer.from("same wrong bytes\n")];
    assert.deepEqual((rows[0]!.packet as Obj)[field], (rows[1]!.packet as Obj)[field]);
    assert.throws(() => checkPackets("factory", marker(TESTS[0].marker, rows), native, fixture));
  }
});

// Whole verbatim Cargo records from retained 41396672119; no product execution.
const actualDefaultReleaseRows: Obj[] =
  String.raw`{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#corsa_runtime@2.0.0-beta.4","manifest_path":"/home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/corsa_runtime-2.0.0-beta.4/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"corsa_runtime","src_path":"/home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/corsa_runtime-2.0.0-beta.4/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"3","debuginfo":0,"debug_assertions":false,"overflow_checks":false,"test":false},"features":[],"filenames":["/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/deps/libcorsa_runtime-54a7ad8e00d29b17.rlib","/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/deps/libcorsa_runtime-54a7ad8e00d29b17.rmeta"],"executable":null,"fresh":false}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#corsa_jsonrpc@2.0.0-beta.4","manifest_path":"/home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/corsa_jsonrpc-2.0.0-beta.4/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"corsa_jsonrpc","src_path":"/home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/corsa_jsonrpc-2.0.0-beta.4/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"3","debuginfo":0,"debug_assertions":false,"overflow_checks":false,"test":false},"features":[],"filenames":["/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/deps/libcorsa_jsonrpc-96521738a65684ad.rlib","/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/deps/libcorsa_jsonrpc-96521738a65684ad.rmeta"],"executable":null,"fresh":false}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#corsa_lsp@2.0.0-beta.4","manifest_path":"/home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/corsa_lsp-2.0.0-beta.4/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"corsa_lsp","src_path":"/home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/corsa_lsp-2.0.0-beta.4/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"3","debuginfo":0,"debug_assertions":false,"overflow_checks":false,"test":false},"features":[],"filenames":["/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/deps/libcorsa_lsp-4ded68e87af789dc.rlib","/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/deps/libcorsa_lsp-4ded68e87af789dc.rmeta"],"executable":null,"fresh":false}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#corsa@2.0.0-beta.4","manifest_path":"/home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/corsa-2.0.0-beta.4/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"corsa","src_path":"/home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/corsa-2.0.0-beta.4/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"3","debuginfo":0,"debug_assertions":false,"overflow_checks":false,"test":false},"features":[],"filenames":["/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/deps/libcorsa-be1f6c606badd802.rlib","/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/deps/libcorsa-be1f6c606badd802.rmeta"],"executable":null,"fresh":false}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#corsa_orchestrator@2.0.0-beta.4","manifest_path":"/home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/corsa_orchestrator-2.0.0-beta.4/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"corsa_orchestrator","src_path":"/home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/corsa_orchestrator-2.0.0-beta.4/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"3","debuginfo":0,"debug_assertions":false,"overflow_checks":false,"test":false},"features":[],"filenames":["/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/deps/libcorsa_orchestrator-ffc733bd7594a35b.rlib","/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/deps/libcorsa_orchestrator-ffc733bd7594a35b.rmeta"],"executable":null,"fresh":false}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#corsa_core@2.0.0-beta.4","manifest_path":"/home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/corsa_core-2.0.0-beta.4/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"corsa_core","src_path":"/home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/corsa_core-2.0.0-beta.4/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"3","debuginfo":0,"debug_assertions":false,"overflow_checks":false,"test":false},"features":[],"filenames":["/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/deps/libcorsa_core-9552adb46a27889f.rlib","/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/deps/libcorsa_core-9552adb46a27889f.rmeta"],"executable":null,"fresh":false}
{"reason":"compiler-artifact","package_id":"registry+https://github.com/rust-lang/crates.io-index#corsa_client@2.0.0-beta.4","manifest_path":"/home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/corsa_client-2.0.0-beta.4/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"corsa_client","src_path":"/home/runner/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/corsa_client-2.0.0-beta.4/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"3","debuginfo":0,"debug_assertions":false,"overflow_checks":false,"test":false},"features":[],"filenames":["/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/deps/libcorsa_client-062f7a2963fe5c00.rlib","/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/deps/libcorsa_client-062f7a2963fe5c00.rmeta"],"executable":null,"fresh":false}
{"reason":"compiler-artifact","package_id":"path+file:///home/runner/_work/vize/vize/crates/vize_canon#0.474.0","manifest_path":"/home/runner/_work/vize/vize/crates/vize_canon/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"vize_canon","src_path":"/home/runner/_work/vize/vize/crates/vize_canon/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"3","debuginfo":0,"debug_assertions":false,"overflow_checks":false,"test":false},"features":["native"],"filenames":["/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/deps/libvize_canon-b5db702dfa644f92.rlib","/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/deps/libvize_canon-b5db702dfa644f92.rmeta"],"executable":null,"fresh":false}
{"reason":"compiler-artifact","package_id":"path+file:///home/runner/_work/vize/vize/crates/vize#0.474.0","manifest_path":"/home/runner/_work/vize/vize/crates/vize/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"vize","src_path":"/home/runner/_work/vize/vize/crates/vize/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"3","debuginfo":0,"debug_assertions":false,"overflow_checks":false,"test":false},"features":["default","glyph","maestro"],"filenames":["/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/libvize.rlib","/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/deps/libvize-30ac51e421ae4f26.rmeta"],"executable":null,"fresh":false}
{"reason":"compiler-artifact","package_id":"path+file:///home/runner/_work/vize/vize/crates/vize_maestro#0.474.0","manifest_path":"/home/runner/_work/vize/vize/crates/vize_maestro/Cargo.toml","target":{"kind":["lib"],"crate_types":["lib"],"name":"vize_maestro","src_path":"/home/runner/_work/vize/vize/crates/vize_maestro/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true},"profile":{"opt_level":"3","debuginfo":0,"debug_assertions":false,"overflow_checks":false,"test":false},"features":["default","glyph","native","retained-program"],"filenames":["/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/deps/libvize_maestro-c41d9a15b26484a9.rlib","/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/deps/libvize_maestro-c41d9a15b26484a9.rmeta"],"executable":null,"fresh":false}
{"reason":"compiler-artifact","package_id":"path+file:///home/runner/_work/vize/vize/crates/vize#0.474.0","manifest_path":"/home/runner/_work/vize/vize/crates/vize/Cargo.toml","target":{"kind":["bin"],"crate_types":["bin"],"name":"vize","src_path":"/home/runner/_work/vize/vize/crates/vize/src/main.rs","edition":"2024","doc":true,"doctest":false,"test":true},"profile":{"opt_level":"3","debuginfo":0,"debug_assertions":false,"overflow_checks":false,"test":false},"features":["default","glyph","maestro"],"filenames":["/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/vize"],"executable":"/home/runner/_work/_temp/initial-canon-target-41396672119-1/release/vize","fresh":false}
{"reason":"build-finished","success":true}`
    .split("\n")
    .map((line) => JSON.parse(line));
const releaseRepo = "/home/runner/_work/vize/vize";
const releaseTarget = "/home/runner/_work/_temp/initial-canon-target-41396672119-1";
await test("actual default CLI records retain native Canon and Maestro dependency contexts", () => {
  const selected = defaultCliArtifact(actualDefaultReleaseRows, releaseTarget, releaseRepo);
  assert.deepEqual(selected.features, ["default", "glyph", "maestro"]);
  assert.equal(selected.executable, `${releaseTarget}/release/vize`);
});
await test("wrong CLI, missing dependency native, foreign contexts and profiling refuse", () => {
  const mutations = [
    ["vize", "features", ["default", "glyph", "maestro", "native"]],
    ["vize", "features", ["default", "glyph"]],
    ["vize_canon", "features", []],
    ["vize_maestro", "features", ["default", "glyph", "retained-program"]],
    ["vize_canon", "features", ["native", "profiling"]],
    ["corsa_core", "features", ["micro-profiling"]],
    ["vize_maestro", "manifest_path", "/foreign/Cargo.toml"],
    ["vize_canon", "target", { name: "vize_canon", kind: ["bin"] }],
    ["vize_maestro", "profile", { opt_level: "0", test: false, debug_assertions: false }],
    ["vize", "profile", { opt_level: "3", test: true, debug_assertions: false }],
  ] as const;
  for (const [name, key, value] of mutations) {
    const rows = structuredClone(actualDefaultReleaseRows);
    const selected = rows.find(
      (row) =>
        (row.target as Obj | undefined)?.name === name &&
        (name !== "vize" || JSON.stringify((row.target as Obj).kind) === '["bin"]'),
    )!;
    selected[key] = value;
    assert.throws(() => defaultCliArtifact(rows, releaseTarget, releaseRepo), name + ":" + key);
  }
});
