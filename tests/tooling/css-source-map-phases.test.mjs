import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  selectPhaseLibraries,
  selectPhaseDirectory,
  phaseDependencyPlan,
  phaseOutputAbsent,
} from "../../tools/benchmarks/scripts/css-source-map-phases.mjs";
import {
  profileModes,
  profileVectors,
  validateNativeTimingPlan,
} from "../../tools/benchmarks/scripts/css-source-map-profile.mjs";

await test("isolated phase driver requires the sole actual rlibs and rejects missing or ambiguous producers", () => {
  const names = ["vize_atelier_sfc", "vize_croquis", "vize_carton", "serde_json"],
    entries = names.map((name) => `lib${name}-1234.rlib`);
  assert.deepEqual(
    selectPhaseLibraries(entries),
    Object.fromEntries(names.map((name, index) => [name, entries[index]])),
  );
  for (const name of names) {
    assert.throws(() =>
      selectPhaseLibraries(entries.filter((file) => file !== `lib${name}-1234.rlib`)),
    );
    assert.throws(() => selectPhaseLibraries([...entries, `lib${name}-5678.rlib`]));
  }
});
await test("phase outputs must start absent and reused binaries cannot acquire fresh output custody", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-phase-custody-"));
  try {
    const file = path.join(directory, "driver");
    phaseOutputAbsent(file);
    fs.writeFileSync(file, "retained output");
    assert.throws(() => phaseOutputAbsent(file));
    const dangling = path.join(directory, "dangling");
    fs.symlinkSync(path.join(directory, "missing"), dangling);
    assert.throws(() => phaseOutputAbsent(dangling));
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
await test("source phases use exactly the unchanged native profile vectors, counts and bytes", () => {
  assert.deepEqual(profileVectors(), [
    { id: "two-rules", source: ".雪😀{color:red}.b{opacity:.5}", batch: 500 },
    {
      id: "400-rules",
      source: Array.from(
        { length: 400 },
        (_, i) => `.r${i}雪😀{opacity:${i / 1000};background:url('../asset.png')}`,
      ).join("\r\n"),
      batch: 10,
    },
    {
      id: "400-bindings",
      source: Array.from(
        { length: 400 },
        (_, i) => `.r${i}雪😀{width:v-bind(width${i});opacity:${i / 1000}}`,
      ).join("\r\n"),
      batch: 10,
    },
  ]);
});

await test("phase discovery distinguishes actual target layouts from host dependencies without ambiguous fallback", () => {
  const entries = ["vize_atelier_sfc", "vize_croquis", "vize_carton", "serde_json"].map(
    (name) => `lib${name}-1234.rlib`,
  );
  for (const directory of ["/owned/ci-opt/deps", "/owned/x86_64-unknown-linux-gnu/ci-opt/deps"]) {
    const owner = { directory, entries };
    assert.deepEqual(
      selectPhaseDirectory([
        { directory: "/owned/host/ci-opt/deps", entries: ["libserde_json-host.rlib"] },
        owner,
      ]),
      { directory, libraries: selectPhaseLibraries(entries) },
    );
    assert.throws(() =>
      selectPhaseDirectory([owner, { directory: "/owned/other/ci-opt/deps", entries }]),
    );
    assert.throws(() => selectPhaseDirectory([{ ...owner, entries: entries.slice(0, 1) }]));
  }
  assert.throws(() =>
    selectPhaseDirectory([
      { directory: "/owned/ci-opt/deps", entries: ["libserde_json-host.rlib"] },
    ]),
  );
});

await test("driver dependency plan keeps actual target libraries and same-role host proc macros together", () => {
  const entries = ["vize_atelier_sfc", "vize_croquis", "vize_carton", "serde_json"].map(
    (name) => `lib${name}-1234.rlib`,
  );
  const host = { directory: "/owned/ci-opt/deps", entries: ["libserde_derive-host.so"] },
    native = { directory: "/owned/x86_64-unknown-linux-gnu/ci-opt/deps", entries };
  assert.deepEqual(phaseDependencyPlan("/owned", [host, native], "x86_64-unknown-linux-gnu"), {
    directory: native.directory,
    libraries: selectPhaseLibraries(entries),
    triple: "x86_64-unknown-linux-gnu",
    searchDirectories: [native.directory, host.directory],
  });
  assert.throws(() => phaseDependencyPlan("/owned", [native], "x86_64-unknown-linux-gnu"));
  assert.throws(() => phaseDependencyPlan("/foreign", [host, native], "x86_64-unknown-linux-gnu"));
  assert.deepEqual(
    phaseDependencyPlan("/owned", [{ ...host, entries }], "x86_64-unknown-linux-gnu")
      .searchDirectories,
    [host.directory],
  );
});

await test("isolated native processes can never schedule map-on calls or shrink interleaved coverage", () => {
  for (let sample = 0; sample < 20; sample++) {
    assert.deepEqual(profileModes("isolated-off", sample), [false]);
    assert.deepEqual(
      profileModes("interleaved", sample),
      sample % 2 === 0 ? [false, true] : [true, false],
    );
  }
  assert.throws(() => profileModes("foreign"));
});

await test("native timing admission retains every vector, mode and batch and refuses missing or invented samples", () => {
  for (const mode of ["interleaved", "isolated-off"]) {
    const packet = {
      measurementMode: mode,
      timings: profileVectors().map((row) => ({
        ...row,
        samples: Array.from({ length: 20 }, () =>
          profileModes(mode).map((sourceMap) => ({
            sourceMap,
            calls: row.batch,
            elapsedNs: 1,
            bytes: 1,
          })),
        ).flat(),
      })),
    };
    validateNativeTimingPlan(packet, mode);
    for (const mutate of [
      (value) => value.timings.pop(),
      (value) => value.timings[0].samples.pop(),
      (value) => {
        value.timings[0].source = "foreign";
      },
      (value) => {
        value.timings[0].samples[0].calls = 0;
      },
      (value) => {
        value.timings[0].samples[0].elapsedNs = 0;
      },
      (value) => {
        value.timings[0].samples[0].sourceMap = !value.timings[0].samples[0].sourceMap;
      },
    ]) {
      const changed = structuredClone(packet);
      mutate(changed);
      assert.throws(() => validateNativeTimingPlan(changed, mode));
    }
  }
});
