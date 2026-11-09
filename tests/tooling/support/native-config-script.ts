import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";

// These complete postimages are independently required by Carton's native
// runtime_source unit. Both observers execute that same dependency-free body.
const pins = {
  validator: "e7cd857464b449579d288fd20e75549875404389200fa40a1bd9517ec4e2ab83",
  runtime: "d109f34087462fb72de92e129a1ad6651de1b2cf4190e68eacb7d8981875e970",
  assembled: "8631ccde19cdda4bf30a248d7d954ad77739f6007468cb631863eb5862d47bf2",
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
