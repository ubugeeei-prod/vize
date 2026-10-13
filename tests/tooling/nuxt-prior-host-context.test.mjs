import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  adaptPriorNuxtHostContext,
  preparePriorNuxtHostContext,
  priorNuxtConstructor,
  priorNuxtSource,
} from "../../tools/support/compat/nuxt/prior-host-context.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const fixture = path.join(
  root,
  "tests/_fixtures/differential/config/vue-config-8371/prior-nuxt-host-context",
);
const original = fs.readFileSync(path.join(fixture, "original-constructor.ts.fixture"));
const sha = (body) => createHash("sha256").update(body).digest("hex");
const before =
  "      vize({ ...compilerOptions, nuxtPageMeta: true, ssrModuleIdRoot: nuxt.options.srcDir }),";
const after =
  '      vize({ ...compilerOptions, nuxtPageMeta: true, ssrModuleIdRoot: nuxt.options.srcDir }, {}, "nuxt"),';

void test("only the whole pinned historical Nuxt constructor receives its private host context", () => {
  const adapted = adaptPriorNuxtHostContext(original, priorNuxtSource);
  assert.equal(original.length, 24345);
  assert.equal(sha(original), "858c488fef903831fa4dcf19c1a534f71dbfc0a75b5e6ef2b7474a49913eee44");
  assert.equal(original.toString().split(before).length, 2);
  assert.deepEqual(adapted, Buffer.from(original.toString().replace(before, after)));
  assert.deepEqual(Buffer.from(adapted.toString().replace(after, before)), original);
  assert.throws(() => adaptPriorNuxtHostContext(original, "f".repeat(40)));
  assert.throws(() => adaptPriorNuxtHostContext(original.toString(), priorNuxtSource));
  for (const changed of [
    Buffer.concat([original, Buffer.from("\n")]),
    original.subarray(1),
    Buffer.from(original.toString().replace("usesVizeCompiler", "usesFakeCompiler")),
    adapted,
  ])
    assert.throws(() => adaptPriorNuxtHostContext(changed, priorNuxtSource));
});

void test("the actual source preparation retains both complete bodies and its explicit adapted identity", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-prior-host-context-"));
  try {
    const source = path.join(directory, "source");
    const artifacts = path.join(directory, "artifacts");
    const target = path.join(source, priorNuxtConstructor);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.mkdirSync(artifacts);
    fs.writeFileSync(target, original);
    preparePriorNuxtHostContext(source, priorNuxtSource, artifacts);
    const adapted = fs.readFileSync(target);
    assert.deepEqual(
      fs.readFileSync(path.join(artifacts, "prior-nuxt-host-context.before.ts")),
      original,
    );
    assert.deepEqual(
      fs.readFileSync(path.join(artifacts, "prior-nuxt-host-context.after.ts")),
      adapted,
    );
    const receipt = JSON.parse(
      fs.readFileSync(path.join(artifacts, "prior-nuxt-host-context.json")),
    );
    assert.deepEqual(receipt.before, { bytes: original.length, sha256: sha(original) });
    assert.deepEqual(receipt.after, { bytes: adapted.length, sha256: sha(adapted) });
    assert.equal(receipt.source, priorNuxtSource);
    assert.equal(receipt.path, priorNuxtConstructor);
    assert.equal(receipt.originalAlgorithmWholeInverse, true);
    assert.equal(
      receipt.credit,
      "historical adapter compatibility only; no current production or consumer acceptance",
    );
    assert.throws(() => preparePriorNuxtHostContext(source, priorNuxtSource, artifacts));
    assert.deepEqual(fs.readFileSync(target), adapted);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

void test("linked, executable and directory source substitutions are refused without adaptation", () => {
  for (const kind of ["link", "executable", "directory"]) {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-prior-host-refusal-"));
    try {
      const source = path.join(directory, "source");
      const artifacts = path.join(directory, "artifacts");
      const target = path.join(source, priorNuxtConstructor);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.mkdirSync(artifacts);
      if (kind === "directory") fs.mkdirSync(target);
      else if (kind === "link") {
        const actual = path.join(directory, "actual.ts");
        fs.writeFileSync(actual, original);
        fs.symlinkSync(actual, target);
      } else fs.writeFileSync(target, original, { mode: 0o755 });
      assert.throws(() => preparePriorNuxtHostContext(source, priorNuxtSource, artifacts), kind);
      assert.deepEqual(fs.readdirSync(artifacts), []);
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  }
});

void test("the original driver, prefetch algorithm and complete failed prior log remain literal", () => {
  const prior = fs.readFileSync(path.join(fixture, "original-driver.mjs.fixture"));
  assert.equal(prior.length, 9227);
  assert.equal(sha(prior), "179937ba6f225bf4ad1d9b8c8b1f46d46c9fa2a45c339602e9533ca6256045ad");
  const imported = 'import { preparePriorNuxtHostContext } from "./prior-host-context.mjs";\n';
  const importAnchor =
    'import { assertPrefetchManifest, observePrefetchManifest } from "./prefetch-manifest-observe.mjs";\n';
  const call = "preparePriorNuxtHostContext(priorRoot, prior, artifacts);\n";
  const callAnchor = 'execFileSync("tar", ["-xf", priorArchive, "-C", priorRoot]);\n';
  assert.equal(prior.toString().split(importAnchor).length, 2);
  assert.equal(prior.toString().split(callAnchor).length, 2);
  const expected = prior
    .toString()
    .replace(importAnchor, importAnchor + imported)
    .replace(callAnchor, callAnchor + call);
  assert.deepEqual(
    fs.readFileSync(path.join(root, "tools/support/compat/nuxt/prefetch-manifest-build.mjs")),
    Buffer.from(expected),
  );
  const failure = fs.readFileSync(path.join(fixture, "original-prior-generate.log.fixture"));
  assert.equal(failure.length, 225);
  assert.equal(sha(failure), "2b806a68e888bc885ec73c3e6254d2744921a715e422809210c4e985236ef9a2");
  assert.match(failure.toString(), /Nuxt build error: TypeError: vue: unknown option script/);
});
