import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { test } from "node:test";
import { nativeLoaderProgram } from "../../tools/support/release/human_reports_11252_11253_public/native-loader-program.ts";
import { originalNativeLoaderModule } from "./support/native-project-export-source.ts";

for (const member of nativeLoaderProgram.modules) {
  test(`the original and current ${member.name} retain the complete authenticated loader law`, () => {
    const original = execFileSync("git", [
      "show",
      `${nativeLoaderProgram.sources[0]}:npm/native/${member.name}`,
    ]);
    const current = fs.readFileSync(new URL(`../../npm/native/${member.name}`, import.meta.url));
    assert.deepEqual(originalNativeLoaderModule(member.name, original), original);
    assert.deepEqual(originalNativeLoaderModule(member.name, current), original);
    assert.throws(() =>
      originalNativeLoaderModule(member.name, Buffer.concat([current, Buffer.from("\n")])),
    );
  });
}
test("unknown exports and modules cannot acquire the original loader authority", () => {
  const body = fs.readFileSync(new URL("../../npm/native/index.js", import.meta.url));
  for (const change of [
    "typeProjectInputs",
    "typeProjectOwners",
    "TypeResolutionBatch",
    "beginTypeResolutionBatch",
    "formatFile",
    "formatFileSync",
    "nuxtTemplateDiagnostics",
  ]) {
    assert.throws(() =>
      originalNativeLoaderModule(
        "index.js",
        Buffer.from(body.toString("utf8").replace(change, "otherProjectApi")),
      ),
    );
  }
  assert.throws(() => originalNativeLoaderModule("unknown.js", body));
});

test("relocated project API exports cannot inherit the original initialization law", () => {
  const current = fs.readFileSync(new URL("../../npm/native/index.js", import.meta.url), "utf8");
  const addition =
    "module.exports.typeProjectInputs = nativeBinding.typeProjectInputs;\n" +
    "module.exports.typeProjectOwners = nativeBinding.typeProjectOwners;\n";
  const original = current.replace(addition, "");
  assert.notEqual(original, current);
  for (const relocated of [
    addition + original,
    original + addition,
    original.replace("const ", addition + "const "),
  ]) {
    assert.throws(() => originalNativeLoaderModule("index.js", Buffer.from(relocated)));
  }
});

test("source formatting exports require their exact canonical facade slot", () => {
  const current = fs.readFileSync(new URL("../../npm/native/index.js", import.meta.url), "utf8");
  const addition =
    "module.exports.formatFile = nativeBinding.formatFile;\n" +
    "module.exports.formatFileSync = nativeBinding.formatFileSync;\n";
  const without = current.replace(addition, "");
  assert.notEqual(without, current);
  assert.deepEqual(
    originalNativeLoaderModule("index.js", Buffer.from(without)),
    originalNativeLoaderModule("index.js", Buffer.from(current)),
    "the previously recognized facade retains the same original loader authority",
  );
  for (const changed of [
    addition + without,
    without + addition,
    without.replace("const ", addition + "const "),
    current + addition,
    current.replace(addition, addition + addition),
    current.replace("module.exports.formatFileSync = nativeBinding.formatFileSync;\n", ""),
    current.replace("nativeBinding.formatFileSync;", "nativeBinding.formatSfc;"),
    current.replace(
      addition,
      "module.exports.formatFileSync = nativeBinding.formatFileSync;\n" +
        "module.exports.formatFile = nativeBinding.formatFile;\n",
    ),
  ]) {
    assert.throws(() => originalNativeLoaderModule("index.js", Buffer.from(changed)));
  }
});

test("the Nuxt template diagnostics export requires its exact canonical facade slot", () => {
  const current = fs.readFileSync(new URL("../../npm/native/index.js", import.meta.url), "utf8");
  const addition =
    "module.exports.nuxtTemplateDiagnostics = nativeBinding.nuxtTemplateDiagnostics;\n";
  const without = current.replace(addition, "");
  assert.notEqual(without, current);
  assert.deepEqual(
    originalNativeLoaderModule("index.js", Buffer.from(without)),
    originalNativeLoaderModule("index.js", Buffer.from(current)),
    "the previously recognized facade retains the same original loader authority",
  );
  for (const changed of [
    addition + without,
    without + addition,
    without.replace("const ", addition + "const "),
    current + addition,
    current.replace(addition, addition + addition),
    current.replace("nativeBinding.nuxtTemplateDiagnostics;", "nativeBinding.formatSfc;"),
  ]) {
    assert.throws(() => originalNativeLoaderModule("index.js", Buffer.from(changed)));
  }
});

test("native-binding.js keeps its original loading law under the load diagnostics", () => {
  const current = fs.readFileSync(
    new URL("../../npm/native/native-binding.js", import.meta.url),
    "utf8",
  );
  for (const [from, to] of [
    ['const linuxGnuGlibcFloor = "2.17";', 'const linuxGnuGlibcFloor = "2.28";'],
    ["  attemptedTargets = nativeTargets(loadErrors);\n", "  attemptedTargets = [];\n"],
    ["let attemptedTargets = [];\n", ""],
  ]) {
    const changed = current.replace(from, to);
    assert.notEqual(changed, current);
    assert.throws(() => originalNativeLoaderModule("native-binding.js", Buffer.from(changed)));
  }
});
