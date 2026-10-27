import assert from "node:assert/strict";
import { test } from "node:test";
import {
  validateCargoDelta,
  validateIdentity,
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
