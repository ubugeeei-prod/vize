import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { basename, dirname, isAbsolute, join, relative, resolve } from "node:path";

// Only generated JSON benchmark configs are read. Original configs stay intact;
// Vize's supported extends ingestion owns compiler/include/path resolution.
function vueOptions(filename, active = new Set()) {
  const path = resolve(filename);
  assert.ok(!active.has(path), `cyclic benchmark config: ${path}`);
  active.add(path);
  try {
    const raw = JSON.parse(readFileSync(path, "utf8"));
    const parents =
      raw.extends === undefined ? [] : Array.isArray(raw.extends) ? raw.extends : [raw.extends];
    let options = {};
    for (const parent of parents) {
      assert.equal(typeof parent, "string");
      let target;
      if (parent.startsWith(".") || isAbsolute(parent)) {
        target = resolve(dirname(path), parent);
        if (!target.endsWith(".json")) target += ".json";
      } else {
        const require = createRequire(path);
        try {
          target = require.resolve(parent);
        } catch {
          target = require.resolve(`${parent}/tsconfig.json`);
        }
      }
      options = { ...options, ...vueOptions(target, active) };
    }
    assert.ok(
      raw.vueCompilerOptions === undefined ||
        (raw.vueCompilerOptions &&
          typeof raw.vueCompilerOptions === "object" &&
          !Array.isArray(raw.vueCompilerOptions)),
    );
    return { ...options, ...raw.vueCompilerOptions };
  } finally {
    active.delete(path);
  }
}

export function vizeTypecheckConfig(directory, config = "tsconfig.json") {
  const filename = resolve(directory, config);
  const options = vueOptions(filename);
  const strict = options.strictTemplates === true;
  const inherited = (name, fallback) =>
    typeof options[name] === "boolean" ? options[name] : fallback;
  const output = join(dirname(filename), `tsconfig.vize-${basename(filename)}`);
  const overlay = {
    extends: `./${relative(dirname(output), filename).replace(/\\/g, "/")}`,
    vueCompilerOptions: {
      checkUnknownProps: inherited("checkUnknownProps", strict),
      fallthroughAttributes: inherited("fallthroughAttributes", false),
      strictComponentAttrs: inherited("strictComponentAttrs", strict),
    },
  };
  writeFileSync(output, `${JSON.stringify(overlay, null, 2)}\n`);
  return relative(directory, output).replace(/\\/g, "/");
}
