import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const base = new URL("../_fixtures/differential/compiler/", import.meta.url);
const read = (path) => readFileSync(new URL(path, base));
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

void test("third-party Vapor runtime packs retain the original report and every fixture byte", () => {
  const registry = JSON.parse(read("manifest.json"));
  const packs = registry.runtimePacks.filter((row) => row.path.startsWith("vapor-hebilicious-"));
  assert(packs.length > 0);
  for (const row of packs) {
    const manifest = read(row.path);
    assert.equal(hash(manifest), row.sha256);
    const pack = JSON.parse(manifest).cases[0];
    for (const file of pack.inputs.files)
      assert.equal(hash(read(`${pack.inputs.root}/${file.path}`)), file.sha256, file.path);
    const original = read(`${pack.inputs.root}/original-issue.md`).toString();
    const reported = [...original.matchAll(/```vue\n([\s\S]*?)```/gu)][0][1];
    assert.equal(read(`${pack.inputs.root}/reported.vue.txt`).toString(), reported);
    if (pack.inputs.root === "vapor-slot-destructure-8626") {
      const owner = reported
        .split("<!-- SlotConsumer.vue -->\n")[0]
        .replace("<!-- SlotOwner.vue -->\n", "");
      assert.equal(JSON.parse(read(`${pack.inputs.root}/Child.vue.json`)), owner);
    }
    if (pack.inputs.root === "vapor-root-fallthrough-8625") {
      const outer = reported
        .split("<!-- FallthroughInner.vue -->\n")[0]
        .replace('<!-- FallthroughOuter.vue, mounted with { "data-owner": "outer" } -->\n', "");
      assert.equal(JSON.parse(read(`${pack.inputs.root}/App.vue.json`)), outer);
    }
    assert.equal(pack.runtime.nativeHandled, 0);
    assert.equal(pack.runtime.lockedVue, "3.6.0-rc.9");
  }
});

void test("the JSX root successor retains the immutable complete #8441 packet", () => {
  const root = new URL(
    "../_fixtures/differential/typecheck/jsx-nested-native-expressions-8441/",
    import.meta.url,
  );
  const fixture = (path) => readFileSync(new URL(path, root));
  const successor = JSON.parse(fixture("root-fallthrough-8625-successor.json"));
  const baseline = JSON.parse(fixture(successor.originalBaseline));
  const original = fixture(successor.originalOutput);
  const corrected = fixture(successor.successorOutput);
  assert.equal(hash(original), successor.originalOutputSha256);
  assert.equal(hash(corrected), successor.successorOutputSha256);
  assert.equal(hash(fixture(successor.source)), successor.sourceSha256);
  assert.equal(
    baseline.cases.find((row) => row.output === successor.originalOutput).outputSha256,
    successor.originalOutputSha256,
  );
  const expected = JSON.parse(original);
  const component = expected.components.find((row) => row.name === "Stepper");
  const old = "      }, true)\n      return n3";
  assert.equal(component.code.split(old).length, 2);
  component.code = component.code.replace(old, "      }, false)\n      return n3");
  assert.deepEqual(JSON.parse(corrected), expected);
});

void test("KeepAlive coverage uses one explicit successor and preserves every original case", () => {
  const repository = new URL("../../", import.meta.url);
  const file = (path) => readFileSync(new URL(path, repository));
  const authority = JSON.parse(
    read("vapor-root-fallthrough-8625/keep-alive-successor-authority.json"),
  );
  // The recorded successor and controls are immutable fixtures. The live
  // expected file is regenerated when the upstream compiler moves, so it is
  // not pinned; the reported KeepAlive case must still be in it.
  const successor = file(authority.successorAuthority);
  assert.equal(hash(successor), authority.successorAuthoritySha256);
  assert.equal(hash(file(authority.semanticAuthority)), authority.semanticAuthoritySha256);
  assert.equal(authority.originalAuthority, "tests/expected/vapor/component.snap");
  const packet = file(authority.originalAuthority)
    .toString()
    .split("\n===\n")
    .find((row) => /^(?:===\n)?name: KeepAlive\n/u.test(row));
  assert.ok(packet, "the reported KeepAlive case remains in the Vapor expectations");
  assert.ok(packet.includes(`--- INPUT ---\n${authority.source}\n`));
  assert.ok(successor.toString().includes(authority.onlyChange.after));
  assert.equal(authority.source, "<KeepAlive><Component /></KeepAlive>");
  assert.equal(authority.publicationQualification, false);
  const captured = file(authority.currentPublicEmissions);
  assert.equal(hash(captured), authority.currentPublicEmissionsSha256);
  const emissions = JSON.parse(captured);
  assert.equal(emissions.diagnosticOnly, true);
  assert.equal(emissions.publicationQualification, false);
  assert.deepEqual(
    emissions.rows.map((row) => row.version),
    ["3.6.0-rc.9", "3.6.0-rc.10"],
  );
  for (const row of emissions.rows) {
    assert.equal(row.source, authority.source);
    assert.equal(hash(row.source), row.sourceSha256);
    assert.equal(hash(row.code), row.codeSha256);
    assert.ok(row.registry.startsWith("https://registry.npmjs.org/@vue/compiler-vapor/"));
    assert.ok(row.integrity.startsWith("sha512-"));
  }
  assert.equal(
    emissions.rows[0].code.replace(authority.onlyChange.before, authority.onlyChange.after),
    emissions.rows[1].code,
  );
  // The coverage gate may only ratchet up and keeps no known failures.
  const gate = file("tests/vize_test_runner/src/coverage.rs").toString();
  const floor = (name) => Number(new RegExp(`${name}: usize = ([0-9_]+)`, "u").exec(gate)?.[1]);
  assert.ok(floor("MIN_VAPOR_PASSED") >= 117);
  assert.ok(floor("MIN_TOTAL_PASSED") >= 743);
  assert.ok(gate.includes("const KNOWN_FAILURES: &[(&str, &str)] = &[]"));
});
