import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";

// These complete postimages are independently required by Carton's native
// runtime_source unit. Both observers execute that same dependency-free body.
const pins = {
  validator: "e7cd857464b449579d288fd20e75549875404389200fa40a1bd9517ec4e2ab83",
  runtime: "36573f96d303132f683025a1de39c3737b5a6a4d4b1fb5cef4dcad5b599b5006",
  assembled: "5ee8cf36d0a8cae745de7a14cd8d6636db5c5d3023cdc5e020a355e5961a27ce",
} as const;
const validatorImport =
  'import { createVueOptionValidator } from "./vite-runtime-validation.js";\n';
const hash = (value: string) => createHash("sha256").update(value).digest("hex");

/** Read the actual native producer closure without installing a project helper. */
export async function nativeLoaderScript(): Promise<string> {
  const loader = new URL("../../../crates/vize_carton/src/config/loader/js.rs", import.meta.url);
  const source = await readFile(loader, "utf8");
  const includes = [...source.matchAll(/include_str!\("([^"]+)"\)/g)];
  assert.deepEqual(
    includes.map((match) => match[1]),
    ["vite-runtime.mjs", "module-runtime.mjs", "config-inputs.mjs"],
  );
  assert.match(source, /let vite_runtime = runtime_source::vite_runtime\(\)/u);
  assert.match(source, /let script = \[\s*vite_runtime\.as_str\(\),/u);
  const scripts = [...source.matchAll(/r#"([\s\S]*?)"#/g)];
  assert.equal(scripts.length, 1);
  const [bodies, validator] = await Promise.all([
    Promise.all(includes.map((match) => readFile(new URL(match[1]!, loader), "utf8"))),
    readFile(new URL("vite-runtime-validation.js", loader), "utf8"),
  ]);
  const runtime = bodies[0]!;
  assert.equal(hash(runtime), pins.runtime);
  assert.equal(hash(validator), pins.validator);
  assert.equal(runtime.split(validatorImport).length, 2, "exactly one owned import");
  const assembled = validator + "\n" + runtime.replace(validatorImport, "");
  assert.equal(hash(assembled), pins.assembled);
  return [assembled, ...bodies.slice(1), scripts[0]![1]].join("");
}
