import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";

// These complete postimages are independently required by Carton's native
// runtime_source unit. Both observers execute that same dependency-free body.
const pins = {
  validator: "515047133aeba1ec3d34c80b9346a532d00bd0d47e3ce21d9cd0a6fa94a1dd82",
  runtime: "b6be222c3d0f68112e7e3d52e801cf757827b12ad6465d5cb1c754f998f0a98f",
  assembled: "c8dad1baecba626f18b08eacd89236a7667e19c214dddb7b4dc7d559269efca5",
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
