import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
// Reuse only the pure own-descriptor serializer, never its rc.14 provider installer.
import { captureWholeValue } from "./support/sfc-entry-validation-reference.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const inputPath = "crates/vize_atelier_sfc/tests/fixtures/ideographic_if_whitespace.json";
const inputSha = "12f5aa7b64146f3a9dc753b37060f2b82721553599bf8a79a39859954d43acd5";
const sha = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
type Case = {
  id: string;
  expression: string;
  template: string;
  source: string;
  accepted: boolean;
  literal: string | null;
};
type RecordValue = Record<string, unknown>;
function object(value: unknown): asserts value is RecordValue {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
}
function call(run: () => unknown) {
  try {
    return { state: "returned" as const, value: run() };
  } catch (error) {
    return { state: "threw" as const, error };
  }
}
function literalValues(value: unknown, values: string[] = [], seen = new Set<object>()) {
  if (value === null || typeof value !== "object" || seen.has(value)) return values;
  seen.add(value);
  if (Reflect.get(value, "type") === "StringLiteral") {
    const text: unknown = Reflect.get(value, "value");
    assert(typeof text === "string");
    values.push(text);
  }
  for (const child of Object.values(value)) literalValues(child, values, seen);
  return values;
}

test("locked Vue preserves authored ideographic conditional whitespace and rejects invalid syntax", () => {
  const bytes = fs.readFileSync(path.join(root, inputPath));
  assert.equal(sha(bytes), inputSha);
  const input: unknown = JSON.parse(bytes.toString("utf8"));
  object(input);
  assert.equal(input.schema, "vize.compiler.ideographic-if-whitespace.v1");
  assert(Array.isArray(input.cases) && input.cases.length === 6);
  const cases = input.cases as Case[];
  assert.deepEqual(
    cases.map((row) => row.id),
    ["leading", "middle", "trailing", "ascii", "quoted-literal", "invalid"],
  );
  const fromUi = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
  const fromVue = createRequire(fromUi.resolve("vue/package.json"));
  const fromCompiler = createRequire(fromVue.resolve("@vue/compiler-sfc/package.json"));
  const packages = [
    "@vue/compiler-sfc",
    "@vue/compiler-dom",
    "@vue/compiler-core",
    "@vue/shared",
    "@babel/parser",
  ];
  const providers = packages.map((name) => {
    const manifestPath = fromCompiler.resolve(`${name}/package.json`);
    const manifestBytes = fs.readFileSync(manifestPath);
    const manifest: unknown = JSON.parse(manifestBytes.toString("utf8"));
    object(manifest);
    const entry = fromCompiler.resolve(name);
    return {
      name,
      manifestPath,
      manifest,
      manifestSha256: sha(manifestBytes),
      entry,
      entryRealPath: fs.realpathSync(entry),
      entrySha256: sha(fs.readFileSync(entry)),
    };
  });
  console.log("ideographic-if reference provider custody", JSON.stringify(providers));
  for (const provider of providers.filter((row) => row.name.startsWith("@vue/")))
    assert.equal(provider.manifest.version, "3.5.35");
  const compiler = fromCompiler("@vue/compiler-sfc") as typeof import("@vue/compiler-sfc");
  const parser = fromCompiler("@babel/parser") as {
    parse(source: string, options: { sourceType: "module" }): unknown;
  };
  const observations = cases.map((fixture) => {
    const parsed = call(() => compiler.parse(fixture.source, { filename: `${fixture.id}.vue` }));
    const options = {
      source: fixture.template,
      filename: `${fixture.id}.vue`,
      id: "ideographic-if-regression",
      compilerOptions: { mode: "module" as const, prefixIdentifiers: true },
    };
    const compiled = call(() => compiler.compileTemplate(options));
    let syntax: ReturnType<typeof call> | null = null;
    if (compiled.state === "returned") {
      object(compiled.value);
      const code: unknown = compiled.value.code;
      if (typeof code === "string")
        syntax = call(() => parser.parse(code, { sourceType: "module" }));
    }
    const observed = { fixture, options, parsed, compiled, syntax };
    // Complete actual results and errors precede any per-case semantic verdict.
    console.log(
      "ideographic-if whole Vue observation",
      JSON.stringify(captureWholeValue(observed)),
    );
    return observed;
  });
  for (const observed of observations) {
    assert.equal(observed.parsed.state, "returned");
    if (observed.parsed.state !== "returned") continue;
    object(observed.parsed.value);
    assert.deepEqual(observed.parsed.value.errors, []);
    const descriptor: unknown = observed.parsed.value.descriptor;
    object(descriptor);
    const template: unknown = descriptor.template;
    object(template);
    assert.equal(template.content, observed.fixture.template);
    assert.equal(observed.compiled.state, "returned");
    if (observed.compiled.state !== "returned") continue;
    object(observed.compiled.value);
    const errors: unknown = observed.compiled.value.errors;
    assert(Array.isArray(errors));
    if (observed.fixture.accepted) {
      assert.deepEqual(errors, []);
      assert(observed.syntax?.state === "returned");
      const actual = literalValues(observed.syntax.value).filter((value) => value === "\u3000");
      assert.deepEqual(actual, observed.fixture.literal === null ? [] : [observed.fixture.literal]);
    } else {
      // Returning code while reporting a compile error is not Vue acceptance.
      assert.equal(errors.length, 1);
    }
  }
});
