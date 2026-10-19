import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import {
  createNuxtComponentResolver,
  injectNuxtComponentImports,
} from "../../../../npm/framework/nuxt/src/components.ts";
import { createNuxtImportOracle, originalNuxtComponents } from "./component-import-oracle.mjs";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const directory = path.join(
  root,
  "tests/_fixtures/differential/compiler/vuefes-app-component-order",
);
const corpus = JSON.parse(fs.readFileSync(path.join(directory, "corpus.json")));
const full = fs.readFileSync(path.join(directory, corpus.input.file));
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
assert.equal(full.length, corpus.input.bytes);
assert.equal(hash(full), corpus.input.sha256);
const gitBlob = createHash("sha1").update(`blob ${full.length}\0`).update(full).digest("hex");
assert.equal(gitBlob, corpus.input.gitBlob);
const fromVue = createRequire(
  fileURLToPath(new URL("./fixtures/heading-cascade/package.json", import.meta.url)),
);
const fromTests = createRequire(path.join(root, "tests/package.json"));
const compiler = fromVue("@vue/compiler-sfc");
assert.equal(compiler.version, corpus.oracle.compilerSfc);
const { parse } = fromTests("@babel/parser");
const { descriptor } = compiler.parse(full.toString(), { filename: "app/app.vue" });
const components = originalNuxtComponents();
const resolver = createNuxtComponentResolver({ rootDir: root, buildDir: path.join(root, ".nuxt") });
resolver.register(components);
const official = createNuxtImportOracle(components);
const imports = (code) =>
  parse(code, { sourceType: "module", plugins: ["typescript"] })
    .program.body.filter((node) => node.type === "ImportDeclaration")
    .map((node) => ({
      source: node.source.value,
      start: node.start,
      end: node.end,
      specifiers: node.specifiers.map((specifier) => ({
        local: specifier.local.name,
        imported: specifier.imported?.name ?? specifier.type,
      })),
    }));
const receipt = {
  sourceSha: execFileSync("git", ["rev-parse", "HEAD"], { cwd: root }).toString().trim(),
  sourceDiffSha256: hash(execFileSync("git", ["diff", "--binary", "HEAD"], { cwd: root })),
  corpus,
  fullSource: full.toString(),
  components,
  compiler: compiler.version,
  oracleLockSha256: hash(
    fs.readFileSync(
      fileURLToPath(new URL("./fixtures/app-component-order/package-lock.json", import.meta.url)),
    ),
  ),
  runtime: "source transform only; original full38 remains actual app acceptance",
  cases: [],
};
const output = path.join(root, "target/vuefes-ssr-style-dependencies");
fs.mkdirSync(output, { recursive: true });
const save = () =>
  fs.writeFileSync(
    path.join(output, "app-component-order.json"),
    JSON.stringify(receipt, null, 2) + "\n",
  );
for (const [name, input] of [
  ["whole-original-script", descriptor.scriptSetup.content],
  [
    "whole-official-compiled-script",
    compiler.compileScript(descriptor, { id: "full-original-app", genDefaultAs: "__app" }).content,
  ],
]) {
  const native = injectNuxtComponentImports(input, (name) => resolver.resolve(name));
  const reference = (await official.transform(input, path.join(root, "app/app.vue"))).code;
  const observation = {
    name,
    input,
    inputImports: imports(input),
    native,
    nativeImports: imports(native),
    official: reference,
    officialImports: imports(reference),
  };
  receipt.cases.push(observation);
  save();
  const contract = (list) => list.map(({ source, specifiers }) => ({ source, specifiers }));
  assert.deepEqual(
    contract(observation.nativeImports),
    contract(observation.officialImports),
    "entire static import sequence and bindings equal selected pinned Nuxt transform",
  );
  assert(
    observation.nativeImports.findIndex((entry) => entry.source === "~/assets/styles/main.css") <
      observation.nativeImports.findIndex((entry) =>
        entry.specifiers.some((specifier) => specifier.local === "NuxtPage"),
      ),
  );
}
console.log(
  "Whole original VueFes app and compiled script retain official Nuxt4.0.3 import order/bindings.",
);
