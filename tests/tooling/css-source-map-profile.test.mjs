import assert from "node:assert/strict";
import { test } from "node:test";
import {
  validateCargoDelta,
  validateIdentity,
  validateSpecializationPackets,
} from "../../tools/benchmarks/scripts/css-source-map-profile.mjs";

await test("CSS source campaign permits only its actual existing-package additive lock change", () => {
  const base =
    'name = "vize_atelier_sfc"\nversion = "0.474.0"\ndependencies = [\n "vize_oxc_parser",\n]\n';
  const head = base.replace(
    ' "vize_oxc_parser",\n',
    ' "vize_oxc_parser",\n "vize_parcel_sourcemap",\n',
  );
  validateCargoDelta(base, head);
  for (const changed of [
    base,
    head.replace("0.474.0", "0.475.0"),
    head + '\nname = "foreign"\n',
    head.replace("vize_parcel_sourcemap", "foreign"),
  ])
    assert.throws(() => validateCargoDelta(base, changed));
});

await test("CSS source campaign rejects swapped source/role, retained targets, foreign image and token", () => {
  const source = "a".repeat(40),
    token = "b".repeat(32),
    hash = "c".repeat(64);
  const identity = { source, side: "base", token, binarySha256: hash, targetInitiallyEmpty: true };
  validateIdentity(identity, source, "base", token, hash);
  for (const changed of [
    { ...identity, source: "d".repeat(40) },
    { ...identity, side: "head" },
    { ...identity, token: "e".repeat(32) },
    { ...identity, binarySha256: "f".repeat(64) },
    { ...identity, targetInitiallyEmpty: false },
  ])
    assert.throws(() => validateIdentity(changed, source, "base", token, hash));
});

await test("binding campaign requires exactly unchanged complete Cargo.lock and refuses unknown baselines", () => {
  const revision = "3a3c8460f966fddbd8e1be4c083d460b24202f90",
    lock = "exact package identities\n";
  validateCargoDelta(lock, lock, revision);
  assert.throws(() => validateCargoDelta(lock, lock + "foreign", revision));
  assert.throws(() => validateCargoDelta(lock, lock, "f".repeat(40)));
});

await test("map-mode candidate admits only its exact qualified source and unchanged full lock", () => {
  const revision = "281d277f1b22ae7dde9cf86564bde832674c3752",
    lock = "exact package identities\n";
  validateCargoDelta(lock, lock, revision);
  assert.throws(() => validateCargoDelta(lock, lock + "foreign", revision));
  assert.throws(() => validateCargoDelta(lock, lock, revision.slice(0, -1) + "1"));
});

await test("map-mode candidate compares whole maps and results while allowing independent timings", () => {
  const result = { code: ".雪😀{color:red}", map: '{"mappings":"AAAA"}', errors: [] };
  const packets = ["base", "head", "head", "base"].map((side, index) => ({
    identity: { side },
    calls: [
      { source: ".雪😀{color:red}", options: { sourceMap: true }, result: structuredClone(result) },
    ],
    timings: [{ id: "two-rules", outputs: { on: structuredClone(result) }, samples: [index + 1] }],
  }));
  validateSpecializationPackets(packets);
  for (const change of [
    (packet) => {
      packet.calls[0].result.map = '{"mappings":"CAAA"}';
    },
    (packet) => {
      delete packet.calls[0].result.map;
    },
    (packet) => {
      packet.calls[0].result.errors.push("changed");
    },
    (packet) => {
      packet.calls[0].options.sourceMap = false;
    },
    (packet) => {
      packet.timings[0].outputs.on.map = "changed";
    },
    (packet) => {
      packet.timings[0].outputs.on.code += " ";
    },
  ]) {
    const changed = structuredClone(packets);
    change(changed[1]);
    assert.throws(() => validateSpecializationPackets(changed));
  }
  assert.throws(() => validateSpecializationPackets(packets.slice(1)));
});
