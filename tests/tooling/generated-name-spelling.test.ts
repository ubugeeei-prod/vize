// Producers spell generated names only through `vize_l0::generated_name`:
// values `$camel`, types `$Pascal`. The old `__vize_snake` spelling survives
// only where an external consumer reads it (published declarations, the CLI
// globals package, bundler-visible module ids and file names).
import assert from "node:assert/strict";
import { readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { GENERATED_NAMES } from "../../tools/support/compat/generated-names.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

/** Old spellings with a consumer outside the producing binary. */
const EXTERNAL = new Set([
  // Published declaration output and its error stub.
  "__vize_component__",
  "__vize_component",
  // Declared by the published `virtual-ts-globals.d.ts`.
  "__vize_defineProps",
  "__vize_defineEmits",
  "__vize_defineExpose",
  "__vize_defineModel",
  "__vize_defineSlots",
  "__vize_withDefaults",
  "__vize_useTemplateRef",
  // File names and module specifiers of the virtual project and declaration output.
  "__vize_helpers",
  "__vize_vue_modules",
  "__vize_auto_imports",
  "__vize_module_augmentations",
  "__vize_external__",
  "__vize_package_scopes__",
  "__vize_javascript__",
  "__vize_authored_vue_ts__",
  "__vize_authored_vue_ts_alias__",
  "__vize_missing_vue_import__",
  "__vize_style__",
  "__vize_imports__",
  "__vize_internal",
  // Compiled runtime output.
  "__vize_static_",
  "__vize_match",
]);

function* sources(directory: string): Generator<string> {
  for (const entry of readdirSync(directory)) {
    const full = path.join(directory, entry);
    if (statSync(full).isDirectory()) {
      if (/^(?:tests|snapshots|fixtures)$|_tests$/u.test(entry)) continue;
      yield* sources(full);
    } else if (entry.endsWith(".rs") && !/(?:^|_)tests?\.rs$/u.test(entry)) {
      yield full;
    }
  }
}

test("the generated name table spells every name once, with one sigil", () => {
  const spellings = [...GENERATED_NAMES.values()];
  assert.equal(new Set(spellings).size, spellings.length);
  for (const spelling of spellings) assert.match(spelling, /^\$[A-Za-z][A-Za-z0-9]*$/u);
});

test("producers spell no generated name the old way", () => {
  const offenders: string[] = [];
  for (const base of ["crates", "davinci"]) {
    for (const file of sources(path.join(root, base))) {
      const text = readFileSync(file, "utf8");
      const production = text.split(/^#\[cfg\(test\)\]\s*\nmod tests\b/mu)[0]!;
      for (const [name] of production.matchAll(/__vize_[A-Za-z0-9_]*/gu)) {
        if (EXTERNAL.has(name)) continue;
        if (/^__vize_(?:static_|internal|match)/u.test(name)) continue;
        offenders.push(`${path.relative(root, file)}: ${name}`);
      }
    }
  }
  assert.deepEqual(offenders, []);
});
