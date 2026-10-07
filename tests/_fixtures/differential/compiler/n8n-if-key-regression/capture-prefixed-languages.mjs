import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const [bundle, output, licenseDirectory] = process.argv.slice(2);
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const compilerSha256 = "4bd5cbcf9bdae4e4264be4ddb14e416074ad96909c56ba9b604e93d5b76b06ec";
assert.equal(digest(readFileSync(bundle)), compilerSha256);
const compiler = await import(pathToFileURL(bundle).href);
assert.equal(compiler.version, "3.5.26");
const contracts = JSON.parse(
  readFileSync(new URL("./prefixed-language-contracts.json", import.meta.url)),
);
const formStock = JSON.parse(
  readFileSync(new URL("../n8n-helper-props/slot-registration-controls.json", import.meta.url)),
);
const form = formStock.originals.find((row) => row.name === "n8n-FormInput");
const originals = [
  {
    name: "n8n_original_instance_ai",
    source: readFileSync(new URL("./InstanceAiConfirmationPanel.vue.txt", import.meta.url), "utf8"),
  },
  form,
];
const licenses = ["LICENSE.md", "LICENSE_EE.md"].map((name) => {
  const source = readFileSync(join(licenseDirectory, name), "utf8");
  return { name, source, sha256: digest(source) };
});
assert.equal(licenses[0].sha256, form.licenseSha256);
const records = [];
for (const original of originals) {
  const contract = contracts.cases.find((row) => row.name === original.name);
  assert.equal(digest(original.source), contract.sourceSha256);
  const { descriptor, errors } = compiler.parse(original.source);
  assert.deepEqual(errors, []);
  for (const mode of ["function", "module"]) {
    for (const isTS of [false, true]) {
      const compilerOptions = {
        mode,
        prefixIdentifiers: true,
        cacheHandlers: false,
        hoistStatic: true,
        comments: false,
        isTS,
      };
      const options = {
        source: descriptor.template.content,
        filename: `${original.name}.vue`,
        id: "n8n-prefixed-language-control",
        sourceMap: false,
        compilerOptions,
      };
      const result = compiler.compileTemplate(options);
      const completeErrors = result.errors.map((error) => ({
        code: error.code,
        message: error.message,
        loc: error.loc,
        name: error.name,
        stack: error.stack,
      }));
      assert.deepEqual(completeErrors, []);
      const input = mode === "function" ? `(function () {\n${result.code}\n})` : result.code;
      const checked = spawnSync(process.execPath, ["--check", "--input-type=module"], {
        input,
        encoding: "utf8",
      });
      records.push({
        name: original.name,
        options,
        output: { code: result.code, map: result.map, tips: result.tips, errors: completeErrors },
        codeSha256: digest(result.code),
        bareJavaScriptCheck: {
          nodeVersion: process.versions.node,
          input,
          status: checked.status,
          signal: checked.signal,
          stdout: checked.stdout,
          stderr: checked.stderr,
          error: checked.error?.message ?? null,
        },
      });
    }
  }
}
const packet = {
  schema: 1,
  compiler: {
    version: compiler.version,
    sha256: compilerSha256,
    entrypoint: "@vue/compiler-sfc/dist/compiler-sfc.esm-browser.js",
    url: "https://unpkg.com/@vue/compiler-sfc@3.5.26/dist/compiler-sfc.esm-browser.js",
  },
  qualification:
    "Full compile output only. Stock reports zero diagnostics in both languages while bare JavaScript parsing may fail; no runtime or valid-JavaScript credit.",
  licenses,
  originals: originals.map((original) => ({ ...original, sourceSha256: digest(original.source) })),
  records,
};
// Keep whole-source golden records in one JSON row each; no content is truncated.
writeFileSync(output, `${JSON.stringify(packet)}\n`);
console.log(
  JSON.stringify({
    output,
    sha256: digest(readFileSync(output)),
    records: records.map(({ name, options, codeSha256, bareJavaScriptCheck }) => ({
      name,
      options: options.compilerOptions,
      codeSha256,
      bareJavaScriptStatus: bareJavaScriptCheck.status,
    })),
  }),
);
