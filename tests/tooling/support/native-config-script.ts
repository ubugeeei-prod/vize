import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";

// These complete postimages are independently required by Carton's native
// runtime_source unit. Both observers execute that same dependency-free body.
const pins = {
  validator: "e7cd857464b449579d288fd20e75549875404389200fa40a1bd9517ec4e2ab83",
  runtime: "0bb73d5f28813a5045f23cf739256ea408093218202596220b7f43639d9d7b0b",
  assembled: "ca66b887bb3e3a8c090b451ddd5ca74f345c4bddfba2b32794d0e7abebae10b8",
} as const;
// Statically spliced modules and the lazily imported config plugin host, in the
// native config-runtime assembly order (runtime_source.rs `OWNED`, `PLUGIN_HOST`).
const owned = ["vite-runtime-validation.js"] as const;
const pluginHost = ["plugin-canonical.js", "plugin-capabilities.js", "plugin-plan.js"] as const;
const pluginHostImport = 'import("./plugin-host.js")';
const pluginToolsImport = 'import("./plugin-tools.js")';
const toolsUnavailable =
  'Promise.reject(new Error("plugin tools are not part of the config runtime"))';
const hash = (value: string) => createHash("sha256").update(value).digest("hex");

/** Mirror runtime_source.rs: drop owned imports and re-export lists, inline the registry. */
function appendModule(
  module: string,
  names: readonly string[],
  isOwned: boolean,
  registry: string,
): string {
  let output = "";
  let rest = module;
  while (rest.length > 0) {
    const newline = rest.indexOf("\n");
    const lineEnd = newline === -1 ? rest.length : newline + 1;
    if (!rest.startsWith("import ")) {
      const line = rest.slice(0, lineEnd);
      if (!(isOwned && line.startsWith("export {") && line.trimEnd().endsWith("};")))
        output += line;
      rest = rest.slice(lineEnd);
      continue;
    }
    const end = rest.indexOf(";\n");
    assert.ok(end >= 0, "terminated import");
    const statement = rest.slice(0, end + 2);
    rest = rest.slice(end + 2);
    const specifier = statement.slice(statement.lastIndexOf(' from "') + 7).split('"')[0]!;
    if (!specifier.startsWith("./")) output += statement;
    else if (specifier === "./plugin-registry.json") {
      const binding = /^import (\w+) from /u.exec(statement)?.[1];
      assert.ok(binding, "registry binding");
      output += `const ${binding} = ${registry.trimEnd()};\n`;
    } else
      assert.ok(
        names.some((name) => `./${name}` === specifier),
        `owned ${specifier}`,
      );
  }
  return output;
}

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
  const assembler = await readFile(new URL("js/runtime_source.rs", loader), "utf8");
  for (const name of [...owned, ...pluginHost])
    assert.match(assembler, new RegExp(`"\\./${name.replace(".", "\\.")}"`));
  const [bodies, modules, hostModules, registry] = await Promise.all([
    Promise.all(includes.map((match) => readFile(new URL(match[1]!, loader), "utf8"))),
    Promise.all(owned.map((name) => readFile(new URL(name, loader), "utf8"))),
    Promise.all(pluginHost.map((name) => readFile(new URL(name, loader), "utf8"))),
    readFile(new URL("plugin-registry.json", loader), "utf8"),
  ]);
  const runtime = bodies[0]!;
  assert.equal(hash(runtime), pins.runtime);
  assert.equal(hash(modules.at(-1)!), pins.validator);
  const bundle = hostModules
    .map((module) => `${appendModule(module, pluginHost, true, registry)}\n`)
    .join("");
  const body = appendModule(runtime, owned, false, registry)
    .replaceAll(
      pluginHostImport,
      `import("data:text/javascript;charset=utf-8," + encodeURIComponent(${JSON.stringify(bundle)}))`,
    )
    .replaceAll(pluginToolsImport, toolsUnavailable);
  const assembled =
    modules.map((module) => `${appendModule(module, owned, true, registry)}\n`).join("") + body;
  assert.equal(hash(assembled), pins.assembled);
  const script = [assembled, ...bodies.slice(1), scripts[0]![1]].join("");
  // Observers pass this body as one `node -e` argument; Linux caps one at 128 KiB.
  assert.ok(Buffer.byteLength(script) < 128 * 1024, "the config runtime exceeds one argv string");
  return script;
}
