import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { expectedHostCalls } from "./n8n-host-replay.mjs";
import {
  disabledFiles,
  editorPrefix,
  effectiveRules,
  layer,
  manifest,
  replayConfig,
  verifyPhysicalEntry,
} from "./n8n-replay-inputs.mjs";

void test("literal frozen object settings contain no invented active-rule hint", () => {
  const baseline = replayConfig("/owned/plugin.mjs", "baseline");
  assert.deepEqual(baseline.settings, manifest.adoption.settings);
  assert.deepEqual(baseline.extends[0].rules, layer.rules);
  assert.equal(Object.keys(layer.rules).length, 81);
  assert.equal(baseline.settings.vize.rules, undefined);
  assert.equal(baseline.overrides.length, 7);
  assert.ok(baseline.overrides.every((override) => override.settings === undefined));
});

void test("six exact disabled scopes and scalar option reset preserve every other rule", () => {
  assert.equal(disabledFiles.length, 6);
  for (const file of disabledFiles) {
    const effective = effectiveRules(file);
    const disabled = Object.keys(effective).filter((name) => effective[name] === "off");
    assert.equal(disabled.length, 1);
    const adjacent = effectiveRules(file.replace(/\.vue$/u, "Adjacent.vue"));
    assert.equal(Object.values(adjacent).includes("off"), false);
    if (file.startsWith(editorPrefix))
      assert.equal(effective["vize/vue/attribute-hyphenation"], "warn");
    else assert.deepEqual(effective["vize/vue/attribute-hyphenation"], ["error", "always"]);
    for (const [name, value] of Object.entries(manifest.adoption.rules))
      if (
        !disabled.includes(name) &&
        !(file.startsWith(editorPrefix) && name.endsWith("/attribute-hyphenation"))
      )
        assert.deepEqual(effective[name], value);
  }
  assert.deepEqual(effectiveRules("packages/Other.vue"), manifest.adoption.rules);
});

void test("optional effective hints carry the whole final map; shared hints stay shared", () => {
  const shared = replayConfig("/owned/plugin.mjs", "shared");
  assert.deepEqual(shared.settings.vize.rules, manifest.adoption.rules);
  assert.ok(shared.overrides.every((override) => override.settings === undefined));
  const effective = replayConfig("/owned/plugin.mjs", "effective");
  for (const override of effective.overrides) {
    const file = override.files[0].replace("**/*.vue", "Example.vue");
    assert.deepEqual(override.settings.vize.rules, effectiveRules(file));
  }
  // The native map is usable, but host override settings must remain an
  // explicit rejected configuration rather than acquiring positive credit.
  for (const file of ["packages/Other.vue", ...disabledFiles]) {
    assert.equal(expectedHostCalls(file, "effective"), 0);
    assert.equal(expectedHostCalls(file, "effective-original"), 0);
    assert.equal(expectedHostCalls(file, "reference"), disabledFiles.includes(file) ? 50 : 51);
    assert.equal(expectedHostCalls(file, "baseline"), 1);
    assert.equal(expectedHostCalls(file, "shared"), 1);
  }
});

void test("stock source authority disables all Vize layers without dropping native rules", () => {
  const stock = replayConfig("/owned/plugin.mjs", "stock");
  assert.equal(Object.keys(stock.rules).length, 51);
  assert.ok(Object.values(stock.rules).every((severity) => severity === "off"));
  assert.ok(
    stock.overrides.every((override) =>
      Object.values(override.rules).every((severity) => severity === "off"),
    ),
  );
  assert.deepEqual(stock.extends[0].rules, layer.rules);
  assert.throws(() => replayConfig("/owned/plugin.mjs", "invented"));
});

void test("physical input authority rejects changed bytes, symlinks and invalid UTF-8", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-n8n-byte-laws-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const file = path.join(root, "Original.vue");
  const bytes = Buffer.from("<template>日本語🙂</template>\r\n");
  const entry = {
    file: "Original.vue",
    mode: "100644",
    kind: "blob",
    blob: createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex"),
  };
  fs.writeFileSync(file, bytes);
  assert.equal(verifyPhysicalEntry(root, entry).scriptless, true);
  fs.writeFileSync(file, Buffer.concat([bytes, Buffer.from("\n")]));
  assert.throws(() => verifyPhysicalEntry(root, entry), /physical bytes differ/u);
  fs.writeFileSync(file, Buffer.from([0xff]));
  assert.throws(() => verifyPhysicalEntry(root, entry), /exact UTF-8/u);
  fs.unlinkSync(file);
  fs.writeFileSync(path.join(root, "Other.vue"), bytes);
  fs.symlinkSync(path.join(root, "Other.vue"), file);
  assert.throws(() => verifyPhysicalEntry(root, entry));
  assert.throws(() => verifyPhysicalEntry(root, { ...entry, mode: "120000" }));
});
