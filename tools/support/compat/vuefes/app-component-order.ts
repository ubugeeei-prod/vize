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
import {
  createNuxtImportOracle,
  nuxtTransform,
  originalNuxtComponents,
} from "./component-import-oracle.ts";
import {
  readJsonRecord,
  record,
  text,
  count,
  type BabelParse,
  type BabelStatement,
  type CompilerSfc,
} from "./oracle-support.ts";

interface ImportRecord {
  readonly source: string;
  readonly start: number;
  readonly end: number;
  readonly specifiers: readonly { local: string; imported: string }[];
}

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const directory = path.join(
  root,
  "tests/_fixtures/differential/compiler/vuefes-app-component-order",
);
const corpus = readJsonRecord(path.join(directory, "corpus.json"));
const input = record(corpus.input, "corpus.input");
const full = fs.readFileSync(path.join(directory, text(input.file, "corpus.input.file")));
const hash = (bytes: Uint8Array | string): string =>
  createHash("sha256").update(bytes).digest("hex");
assert.equal(full.length, count(input.bytes, "corpus.input.bytes"));
assert.equal(hash(full), text(input.sha256, "corpus.input.sha256"));
const gitBlob = createHash("sha1").update(`blob ${full.length}\0`).update(full).digest("hex");
assert.equal(gitBlob, text(input.gitBlob, "corpus.input.gitBlob"));
const fromVue = createRequire(
  fileURLToPath(new URL("./fixtures/heading-cascade/package.json", import.meta.url)),
);
const fromTests = createRequire(path.join(root, "tests/package.json"));
const compiler: CompilerSfc = fromVue("@vue/compiler-sfc");
assert.equal(compiler.version, record(corpus.oracle, "corpus.oracle").compilerSfc);
const { parse }: { parse: BabelParse } = fromTests("@babel/parser");
const { descriptor } = compiler.parse(full.toString(), { filename: "app/app.vue" });
assert.ok(descriptor.scriptSetup, "the original app has a setup script");
const components = originalNuxtComponents();
const resolver = createNuxtComponentResolver({ rootDir: root, buildDir: path.join(root, ".nuxt") });
resolver.register(components);
const official = createNuxtImportOracle(components);
const isImport = (
  node: BabelStatement,
): node is BabelStatement & Required<Pick<BabelStatement, "source" | "specifiers">> =>
  node.type === "ImportDeclaration" && node.source !== undefined && node.specifiers !== undefined;
const imports = (code: string): ImportRecord[] =>
  parse(code, { sourceType: "module", plugins: ["typescript"] })
    .program.body.filter(isImport)
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
  cases: [] as unknown[],
};
const output = path.join(root, "target/vuefes-ssr-style-dependencies");
fs.mkdirSync(output, { recursive: true });
const save = (): void =>
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
] as const) {
  const native = injectNuxtComponentImports(input, (component) => resolver.resolve(component));
  const reference = await nuxtTransform(official, input, path.join(root, "app/app.vue"));
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
  const contract = (list: readonly ImportRecord[]) =>
    list.map(({ source, specifiers }) => ({ source, specifiers }));
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
