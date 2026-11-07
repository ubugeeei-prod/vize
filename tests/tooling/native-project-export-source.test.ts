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
